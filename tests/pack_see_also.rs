use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_gibo")
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gibo-pack-links-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn command(arguments: &[&str]) -> Output {
    Command::new(bin()).args(arguments).output().unwrap()
}

fn wiki(id: &str, title: &str, see_also: &[&str]) -> String {
    let links = if see_also.is_empty() {
        String::new()
    } else {
        format!(
            "see_also:\n{}\n",
            see_also
                .iter()
                .map(|target| format!("  - {target}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    format!(
        "---\nid: {id}\ntitle: {title}\nsummary: Reusable {title}.\nas_of: 2026-07-30T12:00:00+09:00\n{links}---\n\n# {title}\n\nBody.\n"
    )
}

fn manifest(root: &Path, elements: &[&str]) {
    let elements = elements
        .iter()
        .map(|element| format!("    \"{element}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(
        root.join("gibo-pack.json"),
        format!(
            "{{\n  \"schema_version\": 1,\n  \"kind\": \"wiki\",\n  \"elements\": [\n{elements}\n  ]\n}}\n"
        ),
    )
    .unwrap();
}

#[test]
fn wiki_pack_check_requires_every_see_also_target_in_the_manifest() {
    let root = temp_dir("check-closure");
    let pack = root.join("pack");
    fs::create_dir_all(pack.join("concepts")).unwrap();
    fs::write(
        pack.join("concepts/source.md"),
        wiki("wiki:source", "Source concept", &["wiki:declared-target"]),
    )
    .unwrap();
    manifest(&pack, &["concepts/source.md"]);

    let missing = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains(
        "elements[0] concepts/source.md: see_also target is not included in this Wiki Pack: wiki:declared-target"
    ));

    fs::write(
        pack.join("concepts/target.md"),
        wiki("wiki:declared-target", "Declared target", &[]),
    )
    .unwrap();
    manifest(&pack, &["concepts/source.md", "concepts/target.md"]);

    let closed = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(
        closed.status.success(),
        "{}",
        String::from_utf8_lossy(&closed.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pack_check_reads_a_legacy_guru_maker_manifest() {
    let root = temp_dir("legacy-manifest");
    let pack = root.join("pack");
    fs::create_dir_all(pack.join("concepts")).unwrap();
    fs::write(
        pack.join("concepts/source.md"),
        wiki("wiki:source", "Source concept", &[]),
    )
    .unwrap();
    manifest(&pack, &["concepts/source.md"]);
    fs::rename(pack.join("gibo-pack.json"), pack.join("guru-pack.json")).unwrap();

    let legacy = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(
        legacy.status.success(),
        "{}",
        String::from_utf8_lossy(&legacy.stderr)
    );

    fs::write(pack.join("guru-pack.json"), "not json").unwrap();
    manifest(&pack, &["concepts/source.md"]);
    let preferred = command(&["pack", "check", pack.to_str().unwrap()]);
    assert!(
        !preferred.status.success(),
        "a stray legacy manifest beside gibo-pack.json must not be allowed in the Pack tree"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wiki_pack_create_does_not_follow_or_auto_add_see_also_targets() {
    let root = temp_dir("create-closure");
    let workspace = root.join("workspace");
    let initialized = command(&["init", workspace.to_str().unwrap()]);
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );

    let source = workspace.join("gibo/wiki/source.md");
    let target = workspace.join("gibo/wiki/target.md");
    fs::write(
        &source,
        wiki("wiki:source", "Source concept", &["wiki:target"]),
    )
    .unwrap();
    fs::write(&target, wiki("wiki:target", "Target concept", &[])).unwrap();

    let incomplete = root.join("incomplete-pack");
    let rejected = command(&[
        "pack",
        "create",
        "wiki",
        incomplete.to_str().unwrap(),
        "--element",
        source.to_str().unwrap(),
    ]);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr)
        .contains("see_also target is not included in this Wiki Pack: wiki:target"));
    assert!(!incomplete.exists());

    let complete = root.join("complete-pack");
    let created = command(&[
        "pack",
        "create",
        "wiki",
        complete.to_str().unwrap(),
        "--element",
        source.to_str().unwrap(),
        "--element",
        target.to_str().unwrap(),
    ]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let manifest = fs::read_to_string(complete.join("gibo-pack.json")).unwrap();
    assert!(manifest.contains("\"source.md\""));
    assert!(manifest.contains("\"target.md\""));
    fs::remove_dir_all(root).unwrap();
}
