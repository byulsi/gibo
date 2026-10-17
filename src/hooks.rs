//! Minimal lifecycle hooks shared by supported agent hosts.
//!
//! Hooks are advisory guardrails around native file-edit tools. The Rust
//! Runtime owns path inspection and knowledge validation so host adapters stay
//! thin and deterministic.

use crate::{knowledge, workspace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Allow,
    Block(String),
    Feedback(String),
}

#[derive(Debug, Deserialize)]
struct HookInput {
    cwd: PathBuf,
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: Value,
}

#[derive(Serialize)]
struct PostToolFeedback<'a> {
    decision: &'static str,
    reason: &'a str,
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: PostToolSpecificOutput<'a>,
}

#[derive(Serialize)]
struct PostToolSpecificOutput<'a> {
    #[serde(rename = "hookEventName")]
    hook_event_name: &'static str,
    #[serde(rename = "additionalContext")]
    additional_context: &'a str,
}

pub fn evaluate(event: &str, input: &str) -> Result<Outcome, String> {
    let input: HookInput =
        serde_json::from_str(input).map_err(|error| format!("invalid hook input: {error}"))?;
    let Some(root) = workspace_root(&input.cwd) else {
        return Ok(Outcome::Allow);
    };
    let targets = target_paths(&input, &root);

    match event {
        "pre-tool-use" => pre_tool_use(&root, &targets),
        "post-tool-use" => post_tool_use(&root, &targets),
        other => Err(format!("unknown hook event: {other}")),
    }
}

fn pre_tool_use(root: &Path, targets: &[PathBuf]) -> Result<Outcome, String> {
    let protected = resolved_path(&workspace::pack_directory(root));
    if let Some(target) = targets
        .iter()
        .map(|path| resolved_path(path))
        .find(|path| path == &protected || path.starts_with(&protected))
    {
        let relative = target.strip_prefix(root).unwrap_or(&target);
        return Ok(Outcome::Block(format!(
            "Gibo blocked a direct edit to installed Gibo Pack `{}`. \
             Installed sources are immutable provenance; edit the project-owned copy \
             under `gibo/` instead.",
            relative.display()
        )));
    }
    Ok(Outcome::Allow)
}

fn post_tool_use(root: &Path, targets: &[PathBuf]) -> Result<Outcome, String> {
    let local_memory = resolved_path(&root.join("gibo"));
    let edited_memory = targets.iter().map(|path| resolved_path(path)).any(|path| {
        path.starts_with(&local_memory)
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    });
    if !edited_memory {
        return Ok(Outcome::Allow);
    }

    let result = knowledge::check(root);
    if result.valid {
        return Ok(Outcome::Allow);
    }

    let mut message = format!(
        "Gibo memory validation failed after this edit ({} issue{}). \
         Fix the local Markdown before continuing:",
        result.errors.len(),
        if result.errors.len() == 1 { "" } else { "s" }
    );
    for issue in result.errors.iter().take(10) {
        message.push_str(&format!(
            "\n- {} [{}]: {}",
            issue.path, issue.field, issue.message
        ));
    }
    if result.errors.len() > 10 {
        message.push_str(&format!(
            "\n- … and {} more issue(s); run `gibo knowledge check` for the full list.",
            result.errors.len() - 10
        ));
    }

    let output = PostToolFeedback {
        decision: "block",
        reason: &message,
        hook_specific_output: PostToolSpecificOutput {
            hook_event_name: "PostToolUse",
            additional_context: &message,
        },
    };
    serde_json::to_string(&output)
        .map(Outcome::Feedback)
        .map_err(|error| format!("could not serialize hook feedback: {error}"))
}

fn workspace_root(cwd: &Path) -> Option<PathBuf> {
    std::env::var_os("GIBO_HOOK_WORKSPACE")
        .map(PathBuf::from)
        .filter(|path| workspace::is_workspace(path))
        .or_else(|| workspace::find_workspace(cwd))
}

fn target_paths(input: &HookInput, root: &Path) -> Vec<PathBuf> {
    let cwd = if input.cwd.is_absolute() {
        input.cwd.clone()
    } else {
        root.join(&input.cwd)
    };
    let mut raw_paths = Vec::new();

    match input.tool_name.as_str() {
        "Edit" | "Write" => {
            for key in ["file_path", "filePath", "path"] {
                if let Some(path) = input.tool_input.get(key).and_then(Value::as_str) {
                    raw_paths.push(path.to_owned());
                }
            }
        }
        "apply_patch" => {
            if let Some(patch) = input
                .tool_input
                .get("command")
                .and_then(Value::as_str)
                .or_else(|| input.tool_input.as_str())
            {
                raw_paths.extend(patch_paths(patch));
            }
        }
        _ => {}
    }

    raw_paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| {
            let path = PathBuf::from(trim_wrapping_quotes(path.trim()));
            if path.is_absolute() {
                lexical_normalize(&path)
            } else {
                lexical_normalize(&cwd.join(path))
            }
        })
        .collect()
}

fn patch_paths(patch: &str) -> Vec<String> {
    const PREFIXES: &[&str] = &[
        "*** Add File: ",
        "*** Update File: ",
        "*** Delete File: ",
        "*** Move to: ",
    ];
    patch
        .lines()
        .filter_map(|line| {
            PREFIXES
                .iter()
                .find_map(|prefix| line.strip_prefix(prefix))
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
        })
        .collect()
}

fn trim_wrapping_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if matches!(
            (bytes.first(), bytes.last()),
            (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\''))
        ) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn resolved_path(path: &Path) -> PathBuf {
    let normalized = lexical_normalize(path);
    if let Ok(canonical) = fs::canonicalize(&normalized) {
        return canonical;
    }

    let mut cursor = normalized.as_path();
    let mut missing = Vec::<OsString>::new();
    while !cursor.exists() {
        let Some(name) = cursor.file_name() else {
            return normalized;
        };
        missing.push(name.to_owned());
        let Some(parent) = cursor.parent() else {
            return normalized;
        };
        cursor = parent;
    }

    let mut resolved = fs::canonicalize(cursor).unwrap_or_else(|_| cursor.to_path_buf());
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    lexical_normalize(&resolved)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_all_apply_patch_targets() {
        let patch = "*** Begin Patch\n*** Update File: gibo/wiki/a.md\n\
                     *** Move to: gibo/wiki/b.md\n\
                     *** Delete File: .gibo/packs/wiki/lens.md\n\
                     *** End Patch\n";
        assert_eq!(
            patch_paths(patch),
            [
                "gibo/wiki/a.md",
                "gibo/wiki/b.md",
                ".gibo/packs/wiki/lens.md"
            ]
        );
    }

    #[test]
    fn normalizes_parent_components_without_touching_prefixes() {
        assert_eq!(
            lexical_normalize(Path::new("/work/project/gibo/../.gibo/packs/wiki")),
            PathBuf::from("/work/project/.gibo/packs/wiki")
        );
    }
}
