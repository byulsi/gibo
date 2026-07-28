use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_gurumaker")
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("guru-maker-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn command(arguments: &[&str]) -> Output {
    Command::new(bin()).args(arguments).output().unwrap()
}

fn mapped_command(arguments: &[&str], source: &Path, url: &str) -> Output {
    let mut command = Command::new(bin());
    command.args(arguments);
    command.env("GIT_CONFIG_COUNT", "1");
    command.env(
        "GIT_CONFIG_KEY_0",
        format!("url.file://{}/.insteadOf", source.display()),
    );
    command.env("GIT_CONFIG_VALUE_0", url);
    command.output().unwrap()
}

fn hook(workspace: &Path, event: &str, input: &serde_json::Value) -> Output {
    let mut child = Command::new(bin())
        .args(["hook", event])
        .env("GURUMAKER_HOOK_WORKSPACE", workspace)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(input).unwrap().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn git(root: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn element(kind: &str, id: &str, title: &str, body: &str) -> String {
    let source = if kind == "evidence" {
        "source: https://example.test/source\n"
    } else {
        ""
    };
    format!(
        "---\nid: {id}\ntitle: {title}\nsummary: Reusable {title}.\nas_of: 2026-07-28T12:00:00+09:00\n{source}---\n\n# {title}\n\n{body}\n"
    )
}

fn source_repository(root: &Path, kind: &str, filename: &str, id: &str) -> PathBuf {
    let source = root.join(format!("source-{kind}"));
    fs::create_dir_all(&source).unwrap();
    git(&source, &["init"]);
    git(&source, &["config", "user.email", "tests@example.invalid"]);
    git(&source, &["config", "user.name", "Guru tests"]);
    let element_path = source.join(filename);
    fs::create_dir_all(element_path.parent().unwrap()).unwrap();
    fs::write(
        &element_path,
        element(kind, id, "Pack element", "Initial content."),
    )
    .unwrap();
    fs::write(
        source.join("guru-pack.json"),
        format!(
            "{{\n  \"schema_version\": 1,\n  \"kind\": \"{kind}\",\n  \"elements\": [\"{filename}\"]\n}}\n"
        ),
    )
    .unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-m", "initial"]);
    source
}

#[test]
fn init_creates_singular_memory_and_pack_directories() {
    let root = temp_dir("init");
    let output = command(&["init", root.to_str().unwrap(), "--json"]);
    assert!(output.status.success());
    for directory in ["wiki", "lens", "method", "evidence", "decision"] {
        assert!(root.join("guru-maker").join(directory).is_dir());
    }
    assert!(root.join(".guru-maker/packs").is_dir());
    assert_eq!(
        fs::read_to_string(root.join(".guru-maker/.gitignore")).unwrap(),
        "packs/\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".guru-maker/mode")).unwrap(),
        "auto\n"
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"pack_directory\""));
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"guru_mode\": \"auto\""));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn guru_mode_defaults_to_auto_and_persists_explicit_off() {
    let root = temp_dir("mode");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());

    let initial = command(&["mode", "--workspace", root.to_str().unwrap(), "--json"]);
    assert!(initial.status.success());
    let initial: serde_json::Value = serde_json::from_slice(&initial.stdout).unwrap();
    assert_eq!(initial["mode"], "auto");
    assert_eq!(initial["defaulted"], false);

    let off = command(&[
        "mode",
        "off",
        "--workspace",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(off.status.success());
    assert_eq!(
        fs::read_to_string(root.join(".guru-maker/mode")).unwrap(),
        "off\n"
    );

    assert!(command(&["init", root.to_str().unwrap()]).status.success());
    assert_eq!(
        fs::read_to_string(root.join(".guru-maker/mode")).unwrap(),
        "off\n"
    );

    let invalid = command(&["mode", "manual", "--workspace", root.to_str().unwrap()]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("expected auto or off"));

    fs::write(root.join(".guru-maker/mode"), "manual\n").unwrap();
    let malformed = command(&["mode", "--workspace", root.to_str().unwrap()]);
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("expected auto or off"));

    fs::remove_file(root.join(".guru-maker/mode")).unwrap();
    let legacy = command(&["mode", "--workspace", root.to_str().unwrap(), "--json"]);
    assert!(legacy.status.success());
    let legacy: serde_json::Value = serde_json::from_slice(&legacy.stdout).unwrap();
    assert_eq!(legacy["mode"], "auto");
    assert_eq!(legacy["defaulted"], true);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pack_help_exposes_only_the_new_namespace() {
    assert!(!command(&["plugin", "--help"]).status.success());
    for (subcommand, expected) in [
        ("install", "gurumaker pack install <github-url>"),
        ("list", "gurumaker pack list"),
        ("inspect", "gurumaker pack inspect <wiki|lens|method>"),
        ("update", "gurumaker pack update <wiki|lens|method>"),
        ("remove", "gurumaker pack remove <wiki|lens|method>"),
        (
            "create",
            "gurumaker pack create <wiki|lens|method> <output-dir>",
        ),
        ("check", "gurumaker pack check <path>"),
    ] {
        let output = command(&["pack", subcommand, "--help"]);
        assert!(
            output.status.success(),
            "{subcommand}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
}

#[test]
fn hooks_protect_pack_sources_and_validate_local_elements() {
    let root = temp_dir("hooks");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());
    let pack_file = root.join(".guru-maker/packs/wiki/element.md");
    fs::create_dir_all(pack_file.parent().unwrap()).unwrap();
    fs::write(&pack_file, "# Installed source\n").unwrap();
    let protected = hook(
        &root,
        "pre-tool-use",
        &serde_json::json!({
            "cwd": root,
            "tool_name": "Write",
            "tool_input": {"file_path": pack_file}
        }),
    );
    assert_eq!(protected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&protected.stderr).contains("blocked a direct edit"));

    let local = root.join("guru-maker/wiki/element.md");
    fs::write(&local, "# Missing frontmatter\n").unwrap();
    let invalid = hook(
        &root,
        "post-tool-use",
        &serde_json::json!({
            "cwd": root,
            "tool_name": "Write",
            "tool_input": {"file_path": local}
        }),
    );
    assert!(invalid.status.success());
    let feedback: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(feedback["decision"], "block");

    fs::write(&local, element("wiki", "wiki:element", "Element", "Valid.")).unwrap();
    let valid = hook(
        &root,
        "post-tool-use",
        &serde_json::json!({
            "cwd": root,
            "tool_name": "Write",
            "tool_input": {"file_path": local}
        }),
    );
    assert!(valid.status.success());
    assert!(valid.stdout.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn create_and_check_build_a_single_kind_hierarchical_markdown_pack() {
    let root = temp_dir("create");
    let workspace = root.join("workspace");
    assert!(command(&["init", workspace.to_str().unwrap()])
        .status
        .success());
    let first = workspace.join("guru-maker/lens/first.md");
    let second = workspace.join("guru-maker/lens/quality/second.md");
    fs::create_dir_all(second.parent().unwrap()).unwrap();
    fs::write(&first, element("lens", "lens:first", "First", "One.")).unwrap();
    fs::write(&second, element("lens", "lens:second", "Second", "Two.")).unwrap();
    let pack = root.join("pack");
    let output = command(&[
        "pack",
        "create",
        "lens",
        pack.to_str().unwrap(),
        "--element",
        first.to_str().unwrap(),
        "--element",
        second.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(pack.join("guru-pack.json")).unwrap(),
        "{\n  \"schema_version\": 1,\n  \"kind\": \"lens\",\n  \"elements\": [\n    \"first.md\",\n    \"quality/second.md\"\n  ]\n}\n"
    );
    assert!(pack.join("first.md").is_file());
    assert!(pack.join("quality/second.md").is_file());
    assert!(!pack.join("assets").exists());
    assert!(command(&["pack", "check", pack.to_str().unwrap()])
        .status
        .success());

    let wrong_kind = root.join("wrong-kind");
    let rejected = command(&[
        "pack",
        "create",
        "wiki",
        wrong_kind.to_str().unwrap(),
        "--element",
        first.to_str().unwrap(),
    ]);
    assert!(!rejected.status.success());
    assert!(!wrong_kind.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_accepts_hierarchy_and_rejects_undeclared_static_and_linked_elements() {
    let root = temp_dir("pack-check");
    let pack = root.join("pack");
    fs::create_dir_all(&pack).unwrap();
    fs::create_dir_all(pack.join("industry")).unwrap();
    fs::write(
        pack.join("industry/element.md"),
        element("wiki", "wiki:element", "Element", "Body."),
    )
    .unwrap();
    fs::write(
        pack.join("guru-pack.json"),
        "{\n\"schema_version\":1,\n\"kind\":\"wiki\",\n\"elements\":[\"industry/element.md\"]\n}\n",
    )
    .unwrap();
    assert!(command(&["pack", "check", pack.to_str().unwrap()])
        .status
        .success());

    fs::write(pack.join("extra.txt"), "undeclared").unwrap();
    assert!(!command(&["pack", "check", pack.to_str().unwrap()])
        .status
        .success());
    fs::remove_file(pack.join("extra.txt")).unwrap();
    fs::write(
        pack.join("industry/element.md"),
        "---\nid: wiki:element\ntitle: Element\nsummary: Element\nas_of: 2026-01-01T00:00:00Z\nuses:\n  - method:m\n---\n",
    )
    .unwrap();
    assert!(!command(&["pack", "check", pack.to_str().unwrap()])
        .status
        .success());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        fs::remove_file(pack.join("industry/element.md")).unwrap();
        let outside = root.join("outside.md");
        fs::write(
            &outside,
            element("wiki", "wiki:outside", "Outside", "Body."),
        )
        .unwrap();
        symlink(outside, pack.join("industry/element.md")).unwrap();
        assert!(!command(&["pack", "check", pack.to_str().unwrap()])
            .status
            .success());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_rejects_duplicate_ids_invalid_frontmatter_and_executable_elements() {
    let root = temp_dir("pack-element-validation");
    let pack = root.join("pack");
    fs::create_dir_all(&pack).unwrap();
    let first = pack.join("first.md");
    let second = pack.join("second.md");
    fs::write(
        &first,
        element("method", "method:duplicate", "First", "Body."),
    )
    .unwrap();
    fs::write(
        &second,
        element("method", "method:duplicate", "Second", "Body."),
    )
    .unwrap();
    fs::write(
        pack.join("guru-pack.json"),
        "{\"schema_version\":1,\"kind\":\"method\",\"elements\":[\"first.md\",\"second.md\"]}\n",
    )
    .unwrap();

    let duplicate = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicates element id"));

    fs::write(
        &second,
        "---\nid: method:second\ntitle: Second\nas_of: 2026-07-28T12:00:00+09:00\n---\n",
    )
    .unwrap();
    let invalid_frontmatter = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(!invalid_frontmatter.status.success());
    assert!(String::from_utf8_lossy(&invalid_frontmatter.stderr).contains("summary"));

    fs::write(
        &second,
        element("method", "method:second", "Second", "Body."),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&first).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&first, permissions).unwrap();
        let executable = command(&["pack", "check", pack.to_str().unwrap()]);
        assert!(!executable.status.success());
        assert!(String::from_utf8_lossy(&executable.stderr).contains("must not be executable"));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn three_kinds_install_once_and_seed_only_local_memory() {
    let root = temp_dir("install-three");
    let workspace = root.join("workspace");
    assert!(command(&["init", workspace.to_str().unwrap()])
        .status
        .success());
    for (kind, id) in [
        ("wiki", "wiki:shared"),
        ("lens", "lens:shared"),
        ("method", "method:shared"),
    ] {
        let filename = if kind == "wiki" {
            "industry/element.md"
        } else {
            "element.md"
        };
        let source = source_repository(&root, kind, filename, id);
        let url = format!("https://github.com/acme/{kind}.git");
        let output = mapped_command(
            &[
                "pack",
                "install",
                &url,
                "--workspace",
                workspace.to_str().unwrap(),
            ],
            &source,
            &url,
        );
        assert!(
            output.status.success(),
            "{kind}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(workspace
            .join(".guru-maker/packs")
            .join(kind)
            .join("guru-pack.json")
            .is_file());
        assert!(workspace
            .join("guru-maker")
            .join(kind)
            .join(filename)
            .is_file());
    }
    let list = command(&[
        "pack",
        "list",
        "--workspace",
        workspace.to_str().unwrap(),
        "--json",
    ]);
    assert!(list.status.success());
    let listed: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 3);

    let knowledge = command(&[
        "knowledge",
        "list",
        "--workspace",
        workspace.to_str().unwrap(),
        "--json",
    ]);
    let documents: serde_json::Value = serde_json::from_slice(&knowledge.stdout).unwrap();
    assert_eq!(documents.as_array().unwrap().len(), 3);
    assert!(documents
        .as_array()
        .unwrap()
        .iter()
        .all(|document| document["origin"] == "local"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn install_rejects_an_existing_slot_or_local_memory_without_partial_state() {
    let root = temp_dir("install-conflict");
    let workspace = root.join("workspace");
    assert!(command(&["init", workspace.to_str().unwrap()])
        .status
        .success());
    fs::write(
        workspace.join("guru-maker/wiki/local.md"),
        element("wiki", "wiki:local", "Local", "Body."),
    )
    .unwrap();
    let source = source_repository(&root, "wiki", "element.md", "wiki:pack");
    let url = "https://github.com/acme/wiki.git";
    let output = mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap(),
        ],
        &source,
        url,
    );
    assert!(!output.status.success());
    assert!(!workspace.join(".guru-maker/packs/wiki").exists());
    assert_eq!(
        fs::read_dir(workspace.join(".guru-maker/packs"))
            .unwrap()
            .count(),
        0
    );

    fs::remove_file(workspace.join("guru-maker/wiki/local.md")).unwrap();
    assert!(mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap()
        ],
        &source,
        url,
    )
    .status
    .success());
    assert!(!mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap()
        ],
        &source,
        url,
    )
    .status
    .success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn update_changes_only_the_hidden_source_and_remove_preserves_local_memory() {
    let root = temp_dir("update");
    let workspace = root.join("workspace");
    assert!(command(&["init", workspace.to_str().unwrap()])
        .status
        .success());
    let source = source_repository(&root, "method", "element.md", "method:review");
    let url = "https://github.com/acme/method.git";
    assert!(mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap()
        ],
        &source,
        url,
    )
    .status
    .success());
    let local = workspace.join("guru-maker/method/element.md");
    fs::write(
        &local,
        element(
            "method",
            "method:review",
            "Local review",
            "Local evolution.",
        ),
    )
    .unwrap();
    fs::write(
        source.join("element.md"),
        element(
            "method",
            "method:review",
            "Upstream review",
            "Upstream evolution.",
        ),
    )
    .unwrap();
    git(&source, &["add", "element.md"]);
    git(&source, &["commit", "-m", "update"]);
    let commit = git_stdout(&source, &["rev-parse", "HEAD"]);
    let update = mapped_command(
        &[
            "pack",
            "update",
            "method",
            "--ref",
            &commit,
            "--workspace",
            workspace.to_str().unwrap(),
        ],
        &source,
        url,
    );
    assert!(
        update.status.success(),
        "{}",
        String::from_utf8_lossy(&update.stderr)
    );
    assert!(fs::read_to_string(&local)
        .unwrap()
        .contains("Local evolution."));
    assert!(
        fs::read_to_string(workspace.join(".guru-maker/packs/method/element.md"))
            .unwrap()
            .contains("Upstream evolution.")
    );

    let remove = command(&[
        "pack",
        "remove",
        "method",
        "--workspace",
        workspace.to_str().unwrap(),
    ]);
    assert!(remove.status.success());
    assert!(!workspace.join(".guru-maker/packs/method").exists());
    assert!(local.exists());
    assert!(!mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap()
        ],
        &source,
        url,
    )
    .status
    .success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_pack_update_restores_the_prior_source_and_local_memory() {
    let root = temp_dir("invalid-update");
    let workspace = root.join("workspace");
    assert!(command(&["init", workspace.to_str().unwrap()])
        .status
        .success());
    let source = source_repository(&root, "lens", "element.md", "lens:review");
    let url = "https://github.com/acme/lens.git";
    assert!(mapped_command(
        &[
            "pack",
            "install",
            url,
            "--workspace",
            workspace.to_str().unwrap()
        ],
        &source,
        url,
    )
    .status
    .success());
    let installed = workspace.join(".guru-maker/packs/lens");
    let before = git_stdout(&installed, &["rev-parse", "HEAD"]);
    let local_before = fs::read_to_string(workspace.join("guru-maker/lens/element.md")).unwrap();
    fs::write(
        source.join("guru-pack.json"),
        "{\"schema_version\":1,\"kind\":\"lens\",\"elements\":[\"missing.md\"]}\n",
    )
    .unwrap();
    git(&source, &["add", "guru-pack.json"]);
    git(&source, &["commit", "-m", "invalid"]);
    let invalid = git_stdout(&source, &["rev-parse", "HEAD"]);
    let output = mapped_command(
        &[
            "pack",
            "update",
            "lens",
            "--ref",
            &invalid,
            "--workspace",
            workspace.to_str().unwrap(),
        ],
        &source,
        url,
    );
    assert!(!output.status.success());
    assert_eq!(git_stdout(&installed, &["rev-parse", "HEAD"]), before);
    assert_eq!(
        fs::read_to_string(workspace.join("guru-maker/lens/element.md")).unwrap(),
        local_before
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn knowledge_check_enforces_relationship_kinds_and_preserves_hierarchy() {
    let root = temp_dir("relationships");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());
    fs::write(
        root.join("guru-maker/wiki/w.md"),
        element("wiki", "wiki:w", "Wiki", "Independent."),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/method/m.md"),
        element("method", "method:m", "Method", "Independent."),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/evidence/e.md"),
        element("evidence", "evidence:e", "Evidence", "Observed."),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/decision/d.md"),
        "---\nid: decision:d\ntitle: Decision\nsummary: Decision.\nas_of: 2026-07-28T12:00:00+09:00\nuses:\n  - wiki:w\n  - method:m\nsupports:\n  - evidence:e\n---\n",
    )
    .unwrap();
    assert!(
        command(&["knowledge", "check", "--workspace", root.to_str().unwrap()])
            .status
            .success()
    );

    fs::write(
        root.join("guru-maker/wiki/w.md"),
        "---\nid: wiki:w\ntitle: Wiki\nsummary: Wiki.\nas_of: 2026-07-28T12:00:00+09:00\nuses:\n  - method:m\n---\n",
    )
    .unwrap();
    let invalid = command(&["knowledge", "check", "--workspace", root.to_str().unwrap()]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("relationship is not permitted"));

    fs::write(
        root.join("guru-maker/wiki/w.md"),
        element("wiki", "wiki:w", "Wiki", "Independent."),
    )
    .unwrap();
    fs::create_dir(root.join("guru-maker/wiki/nested")).unwrap();
    fs::write(
        root.join("guru-maker/wiki/nested/child.md"),
        element("wiki", "wiki:child", "Child", "Nested knowledge."),
    )
    .unwrap();
    let nested = command(&["knowledge", "check", "--workspace", root.to_str().unwrap()]);
    assert!(nested.status.success());
    fs::write(root.join("guru-maker/wiki/nested/data.txt"), "not Markdown").unwrap();
    let invalid_file = command(&["knowledge", "check", "--workspace", root.to_str().unwrap()]);
    assert!(!invalid_file.status.success());
    assert!(String::from_utf8_lossy(&invalid_file.stderr)
        .contains("elements must be non-symlink Markdown"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn decision_graph_contains_only_local_valid_relationships() {
    let root = temp_dir("graph");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());
    fs::write(
        root.join("guru-maker/wiki/w.md"),
        element("wiki", "wiki:w", "Wiki", "Independent."),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/evidence/e.md"),
        element("evidence", "evidence:e", "Evidence", "Observed."),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/decision/d.md"),
        "---\nid: decision:d\ntitle: Decision\nsummary: Decision.\nas_of: 2026-07-28T12:00:00+09:00\nuses:\n  - wiki:w\nsupports:\n  - evidence:e\n---\n",
    )
    .unwrap();
    let output = command(&["graph", root.to_str().unwrap(), "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stats: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stats["edges"], 3);
    assert!(stats.get("plugins").is_none());
    let page = fs::read_to_string(root.join("guru-maker/decision-graph.html")).unwrap();
    assert!(page.contains("\"relationship\":\"uses\""));
    assert!(page.contains("\"relationship\":\"supports\""));
    assert!(!page.contains("plugin:"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retrieval_remains_bounded_and_reports_authored_relationships() {
    let root = temp_dir("retrieval");
    assert!(command(&["init", root.to_str().unwrap()]).status.success());
    fs::write(
        root.join("guru-maker/decision/original.md"),
        element(
            "decision",
            "decision:original",
            "Original",
            "Initial margin thesis.",
        ),
    )
    .unwrap();
    fs::write(
        root.join("guru-maker/decision/revision.md"),
        "---\nid: decision:revision\ntitle: Revision\nsummary: Later margin review.\nas_of: 2026-07-28T12:00:00+09:00\nupdates:\n  - decision:original\ncontradicts:\n  - decision:original\n---\n\n# Revision\nChanged.",
    )
    .unwrap();
    let output = command(&[
        "knowledge",
        "search",
        "decision:original",
        "--workspace",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(output.status.success());
    let results: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let revision = results
        .as_array()
        .unwrap()
        .iter()
        .find(|result| result["id"] == "decision:revision")
        .unwrap();
    assert_eq!(
        revision["relationships"],
        serde_json::json!([
            {"type": "contradicts", "target": "decision:original"},
            {"type": "updates", "target": "decision:original"}
        ])
    );
    let read = command(&[
        "knowledge",
        "read",
        "decision:revision",
        "--section",
        "Revision",
        "--workspace",
        root.to_str().unwrap(),
    ]);
    assert!(read.status.success());
    assert_eq!(String::from_utf8_lossy(&read.stdout).trim(), "Changed.");
    fs::remove_dir_all(root).unwrap();
}
