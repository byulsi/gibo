use crate::{knowledge, workspace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PACK_KINDS: &[&str] = &["wiki", "lens", "method"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub schema_version: u32,
    pub kind: String,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackInfo {
    pub path: String,
    pub repository: Option<String>,
    pub commit: Option<String>,
    pub reference: Option<String>,
    pub manifest: PackManifest,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub path: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub info: Option<PackInfo>,
}

#[derive(Debug, Clone)]
pub struct GitHubRepository {
    pub repo: String,
    pub clone_url: String,
}

pub fn normalize_github_url(value: &str) -> Result<GitHubRepository, String> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(
            "GitHub repository URL must be a non-empty string without surrounding whitespace"
                .into(),
        );
    }
    let (repo, clone_url) = if let Some(path) = value.strip_prefix("https://github.com/") {
        if path.contains('?') || path.contains('#') || path.contains('@') {
            return Err(
                "GitHub repository URL must not contain credentials, a query, or a fragment".into(),
            );
        }
        let (owner, repo) = split_owner_repo(path)?;
        (
            repo.clone(),
            format!("https://github.com/{owner}/{repo}.git"),
        )
    } else if let Some(path) = value.strip_prefix("git@github.com:") {
        let (owner, repo) = split_owner_repo(path)?;
        (repo.clone(), format!("git@github.com:{owner}/{repo}.git"))
    } else if let Some(path) = value.strip_prefix("ssh://git@github.com/") {
        let (owner, repo) = split_owner_repo(path)?;
        (
            repo.clone(),
            format!("ssh://git@github.com/{owner}/{repo}.git"),
        )
    } else {
        return Err("pack source must be a GitHub HTTPS or SSH repository URL".into());
    };
    Ok(GitHubRepository { repo, clone_url })
}

pub fn pack_target(workspace_root: &Path, kind: &str) -> Result<PathBuf, String> {
    require_pack_kind(kind)?;
    Ok(workspace::pack_directory(workspace_root).join(kind))
}

pub fn validate_manifest(root: &Path) -> Result<PackManifest, String> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|_| "pack root must be an existing non-symlink directory".to_string())?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("pack root must be an existing non-symlink directory".into());
    }
    let manifest_path = root.join("guru-pack.json");
    let metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|_| "pack must contain a non-symlink root guru-pack.json".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("pack must contain a non-symlink root guru-pack.json".into());
    }
    let content = fs::read_to_string(&manifest_path)
        .map_err(|_| "guru-pack.json must be valid UTF-8 JSON".to_string())?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|_| "guru-pack.json must be valid JSON".to_string())?;
    let manifest: PackManifest = serde_json::from_value(value)
        .map_err(|error| format!("guru-pack.json has an invalid shape: {error}"))?;
    if manifest.schema_version != 1 {
        return Err("guru-pack.json schema_version must be integer 1".into());
    }
    require_pack_kind(&manifest.kind)?;
    if manifest.elements.is_empty() {
        return Err("guru-pack.json elements must be a non-empty list".into());
    }

    let mut paths = HashSet::new();
    let mut ids = HashSet::new();
    for (index, element) in manifest.elements.iter().enumerate() {
        validate_element_name(element).map_err(|error| format!("elements[{index}] {error}"))?;
        if !paths.insert(element) {
            return Err(format!("elements[{index}] duplicates {element:?}"));
        }
        validate_element_path(root, element)?;
        let document = knowledge::load_pack_element(&root.join(element), &manifest.kind)?;
        let issues = knowledge::validation_issues(&document);
        if let Some(issue) = issues.first() {
            return Err(format!(
                "elements[{index}] {}: {}: {}",
                issue.path, issue.field, issue.message
            ));
        }
        if !ids.insert(document.id.clone()) {
            return Err(format!(
                "elements[{index}] duplicates element id {:?}",
                document.id
            ));
        }
    }

    let allowed_files = manifest
        .elements
        .iter()
        .map(String::as_str)
        .chain(["guru-pack.json"])
        .collect::<HashSet<_>>();
    let mut allowed_directories = HashSet::new();
    for element in &manifest.elements {
        let mut parent = Path::new(element).parent();
        while let Some(path) = parent.filter(|path| !path.as_os_str().is_empty()) {
            allowed_directories.insert(display(path));
            parent = path.parent();
        }
    }
    validate_pack_tree(root, root, &allowed_files, &allowed_directories)?;
    Ok(manifest)
}

pub fn install_pack(
    workspace_root: &Path,
    source: &str,
    reference: Option<&str>,
) -> Result<PackInfo, String> {
    workspace::require_workspace(workspace_root)?;
    let repository = normalize_github_url(source)?;
    if let Some(reference) = reference {
        validate_ref(reference)?;
    }
    let parent = workspace::pack_directory(workspace_root);
    fs::create_dir_all(&parent).map_err(io_error)?;
    let staged = staging_path(&parent, &format!("source-{}", repository.repo));
    let mut local_stage = None;
    let result = (|| {
        run_git(None, &["clone", &repository.clone_url, &display(&staged)])?;
        if let Some(reference) = reference {
            let resolved = resolve_ref(&staged, reference)?;
            run_git(Some(&staged), &["checkout", "--detach", &resolved])?;
        }
        let manifest = validate_manifest(&staged)?;
        let target = pack_target(workspace_root, &manifest.kind)?;
        if target.exists() || is_symlink(&target)? {
            return Err(format!(
                "{} Pack is already installed in this project",
                manifest.kind
            ));
        }
        let local = workspace_root.join("guru-maker").join(&manifest.kind);
        ensure_empty_directory(&local, &manifest.kind)?;

        let staged_local = staging_path(&parent, &format!("local-{}", manifest.kind));
        fs::create_dir_all(&staged_local).map_err(io_error)?;
        local_stage = Some(staged_local.clone());
        for element in &manifest.elements {
            let destination = staged_local.join(element);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(staged.join(element), destination).map_err(io_error)?;
        }

        fs::remove_dir(&local).map_err(io_error)?;
        if let Err(error) = fs::rename(&staged, &target) {
            fs::create_dir_all(&local).map_err(io_error)?;
            return Err(io_error(error));
        }
        if let Err(error) = fs::rename(&staged_local, &local) {
            let _ = fs::remove_dir_all(&target);
            let _ = fs::create_dir_all(&local);
            return Err(io_error(error));
        }
        local_stage = None;
        match inspect_pack(&target) {
            Ok(info) => Ok(info),
            Err(error) => {
                let _ = fs::remove_dir_all(&target);
                let _ = fs::remove_dir_all(&local);
                let _ = fs::create_dir_all(&local);
                Err(error)
            }
        }
    })();
    if staged.exists() {
        let _ = fs::remove_dir_all(&staged);
    }
    if let Some(path) = local_stage {
        let _ = fs::remove_dir_all(path);
    }
    result
}

pub fn list_packs(workspace_root: &Path) -> Result<Vec<PackInfo>, String> {
    workspace::require_workspace(workspace_root)?;
    let mut packs = Vec::new();
    for kind in PACK_KINDS {
        let path = pack_target(workspace_root, kind)?;
        if path.is_dir() && !is_symlink(&path)? {
            packs.push(inspect_pack_slot(workspace_root, kind)?);
        }
    }
    Ok(packs)
}

pub fn inspect_pack_slot(workspace_root: &Path, kind: &str) -> Result<PackInfo, String> {
    workspace::require_workspace(workspace_root)?;
    let root = pack_target(workspace_root, kind)?;
    let info = inspect_pack(&root)?;
    if info.manifest.kind != kind {
        return Err(format!(
            "{kind} Pack slot contains a {} manifest",
            info.manifest.kind
        ));
    }
    Ok(info)
}

pub fn inspect_pack(root: &Path) -> Result<PackInfo, String> {
    let manifest = validate_manifest(root)?;
    let repository = git_optional(root, &["config", "--get", "remote.origin.url"])?
        .and_then(|value| normalize_github_url(&value).ok())
        .map(|repository| repository.clone_url);
    let commit = git_optional(root, &["rev-parse", "HEAD"])?;
    let branch = git_optional(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let reference = match branch {
        Some(value) => Some(value),
        None => git_optional(root, &["describe", "--tags", "--exact-match", "HEAD"])?,
    };
    Ok(PackInfo {
        path: display(root),
        repository,
        commit,
        reference,
        manifest,
    })
}

pub fn update_pack(root: &Path, reference: Option<&str>) -> Result<PackInfo, String> {
    let before_manifest = validate_manifest(root)?;
    if let Some(reference) = reference {
        validate_ref(reference)?;
    }
    if git_optional(root, &["status", "--porcelain"])?.is_some() {
        return Err("pack checkout has local changes; restore or remove them before update".into());
    }
    let origin = git_optional(root, &["config", "--get", "remote.origin.url"])?
        .ok_or_else(|| "pack checkout has no origin remote".to_string())?;
    normalize_github_url(&origin)
        .map_err(|_| "pack origin must be a GitHub URL without embedded credentials")?;
    let before_commit = run_git(Some(root), &["rev-parse", "HEAD"])?;
    let before_branch = git_optional(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let result = (|| {
        run_git(Some(root), &["fetch", "--tags", "origin"])?;
        if let Some(reference) = reference {
            let resolved = resolve_ref(root, reference)?;
            run_git(Some(root), &["checkout", "--detach", &resolved])?;
        } else if before_branch.is_some() {
            run_git(Some(root), &["merge", "--ff-only", "@{upstream}"])?;
        } else {
            return Err("detached pack checkout requires an explicit ref to update".into());
        }
        let after = inspect_pack(root)?;
        if after.manifest.kind != before_manifest.kind {
            return Err("pack update must not change the installed kind".into());
        }
        Ok(after)
    })();
    match result {
        Ok(info) => Ok(info),
        Err(error) => {
            let restored = if let Some(branch) = before_branch {
                run_git(Some(root), &["checkout", &branch])
                    .and_then(|_| run_git(Some(root), &["reset", "--hard", &before_commit]))
            } else {
                run_git(Some(root), &["checkout", "--detach", &before_commit])
            };
            match restored {
                Ok(_) => Err(error),
                Err(restore_error) => Err(format!(
                    "{error}; failed to restore the prior checkout: {restore_error}"
                )),
            }
        }
    }
}

pub fn remove_pack(workspace_root: &Path, kind: &str) -> Result<String, String> {
    workspace::require_workspace(workspace_root)?;
    let target = pack_target(workspace_root, kind)?;
    if !target.is_dir() || is_symlink(&target)? {
        return Err(format!("{kind} Pack is not installed in this project"));
    }
    fs::remove_dir_all(&target).map_err(io_error)?;
    Ok(display(&target))
}

pub fn create_pack(
    destination: &Path,
    kind: &str,
    elements: &[String],
) -> Result<PackManifest, String> {
    require_pack_kind(kind)?;
    if destination.exists() {
        return Err("pack destination already exists".into());
    }
    if elements.is_empty() {
        return Err("pack create requires at least one explicit --element".into());
    }
    fs::create_dir_all(destination).map_err(io_error)?;
    let result = (|| {
        let mut declared = Vec::new();
        let mut paths = HashSet::new();
        for source in elements {
            let source = Path::new(source);
            let relative = validate_local_element(source, kind)?;
            if !paths.insert(relative.clone()) {
                return Err(format!("element paths must be unique: {relative}"));
            }
            let target = destination.join(&relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(io_error)?;
            }
            fs::copy(source, target).map_err(io_error)?;
            declared.push(relative);
        }
        let manifest = PackManifest {
            schema_version: 1,
            kind: kind.into(),
            elements: declared,
        };
        let rendered =
            serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
        fs::write(destination.join("guru-pack.json"), format!("{rendered}\n")).map_err(io_error)?;
        validate_manifest(destination)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}

pub fn check_pack(path: &Path) -> CheckResult {
    match inspect_pack(path) {
        Ok(info) => CheckResult {
            path: display(path),
            valid: true,
            errors: Vec::new(),
            info: Some(info),
        },
        Err(error) => CheckResult {
            path: display(path),
            valid: false,
            errors: vec![error],
            info: None,
        },
    }
}

fn ensure_empty_directory(path: &Path, kind: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("local {kind} memory directory does not exist"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("local {kind} memory path must be a directory"));
    }
    if fs::read_dir(path).map_err(io_error)?.next().is_some() {
        return Err(format!(
            "cannot install {kind} Pack because local {kind} memory already exists"
        ));
    }
    Ok(())
}

fn validate_local_element(path: &Path, kind: &str) -> Result<String, String> {
    validate_regular_markdown(path)?;
    let collection = path
        .ancestors()
        .skip(1)
        .find(|ancestor| {
            ancestor.file_name().and_then(|value| value.to_str()) == Some(kind)
                && ancestor
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|value| value.to_str())
                    == Some("guru-maker")
        })
        .ok_or_else(|| format!("elements for a {kind} Pack must be under guru-maker/{kind}/"))?;
    validate_no_symlink_components(collection, path)?;
    let document = knowledge::load_pack_element(path, kind)?;
    let issues = knowledge::validation_issues(&document);
    if let Some(issue) = issues.first() {
        return Err(format!(
            "{}: {}: {}",
            issue.path, issue.field, issue.message
        ));
    }
    path.strip_prefix(collection)
        .map(display)
        .map_err(|_| format!("element is outside guru-maker/{kind}/"))
}

fn validate_element_name(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if path.is_absolute()
        || value.contains('\\')
        || value
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some("md")
    {
        return Err("must be a safe repository-relative .md path".into());
    }
    Ok(())
}

fn validate_element_path(root: &Path, relative: &str) -> Result<(), String> {
    let path = root.join(relative);
    validate_no_symlink_components(root, &path)?;
    validate_regular_markdown(&path)
}

fn validate_no_symlink_components(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("path escapes its collection: {}", path.display()))?;
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        cursor.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&cursor)
            .map_err(|_| format!("declared element is missing: {}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "path must not contain symlinks: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_pack_tree(
    root: &Path,
    directory: &Path,
    allowed_files: &HashSet<&str>,
    allowed_directories: &HashSet<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let relative = display(path.strip_prefix(root).unwrap_or(&path));
        if relative == ".git" {
            let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
            if metadata.file_type().is_symlink() {
                return Err("pack Git metadata must not be a symlink".into());
            }
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("pack path must not be a symlink: {relative}"));
        }
        if metadata.is_dir() {
            if !allowed_directories.contains(&relative) {
                return Err(format!("pack contains undeclared directory: {relative}"));
            }
            validate_pack_tree(root, &path, allowed_files, allowed_directories)?;
        } else if !allowed_files.contains(relative.as_str()) {
            return Err(format!("pack contains undeclared file: {relative}"));
        }
    }
    Ok(())
}

fn validate_regular_markdown(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("declared element is missing: {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "element must be a non-symlink regular file: {}",
            path.display()
        ));
    }
    if path.extension().and_then(|value| value.to_str()) != Some("md") {
        return Err(format!("element must be Markdown: {}", path.display()));
    }
    if is_executable(path)? {
        return Err(format!(
            "element must not be executable: {}",
            path.display()
        ));
    }
    Ok(())
}

fn require_pack_kind(kind: &str) -> Result<(), String> {
    if PACK_KINDS.contains(&kind) {
        Ok(())
    } else {
        Err("Pack kind must be one of wiki, lens, method".into())
    }
}

fn split_owner_repo(value: &str) -> Result<(String, String), String> {
    let value = value.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = value.split('/');
    let owner = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default();
    if parts.next().is_some() || !safe_identifier(owner) || !safe_identifier(repo) {
        return Err("GitHub repository URL must identify exactly safe owner/repo".into());
    }
    Ok((owner.to_ascii_lowercase(), repo.to_ascii_lowercase()))
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn validate_ref(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.starts_with('-')
        || value.contains("..")
        || value.contains("@{")
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err("Git ref contains unsafe characters".into());
    }
    Ok(())
}

fn resolve_ref(root: &Path, reference: &str) -> Result<String, String> {
    for candidate in [
        format!("refs/tags/{reference}^{{commit}}"),
        format!("refs/remotes/origin/{reference}^{{commit}}"),
        format!("{reference}^{{commit}}"),
    ] {
        let output = Command::new("git")
            .current_dir(root)
            .args(["rev-parse", "--verify", &candidate])
            .output()
            .map_err(io_error)?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
        }
    }
    Err(format!("Git ref was not found: {reference}"))
}

fn run_git(root: Option<&Path>, arguments: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    if let Some(root) = root {
        command.current_dir(root);
    }
    let output = command.args(arguments).output().map_err(io_error)?;
    git_output(arguments, output)
}

fn git_optional(root: &Path, arguments: &[&str]) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(io_error)?;
    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok((!value.is_empty()).then_some(value))
    } else {
        Ok(None)
    }
}

fn git_output(arguments: &[&str], output: Output) -> Result<String, String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if message.is_empty() {
            format!("git {} failed", arguments.join(" "))
        } else {
            message
        })
    }
}

fn staging_path(parent: &Path, key: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    parent.join(format!(".staging-{key}-{}-{nonce}", std::process::id()))
}

fn is_symlink(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_error(error)),
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    Ok(fs::metadata(path).map_err(io_error)?.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> Result<bool, String> {
    Ok(false)
}

fn display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}
