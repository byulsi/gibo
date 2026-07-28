use serde_json::Value;
use std::{fs, path::Path};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn json(path: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(root().join(path)).unwrap()).unwrap()
}

fn normalized(path: &str) -> String {
    fs::read_to_string(root().join(path))
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cargo_version() -> String {
    fs::read_to_string(root().join("Cargo.toml"))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("Cargo.toml must declare a package version")
        .to_owned()
}

#[test]
fn public_identity_and_runtime_version_are_synchronized() {
    let codex = json(".codex-plugin/plugin.json");
    let codex_marketplace = json(".agents/plugins/marketplace.json");
    let claude = json(".claude-plugin/plugin.json");
    let marketplace = json(".claude-plugin/marketplace.json");
    let runtime_version = fs::read_to_string(root().join("shared/gurumaker/runtime/version"))
        .unwrap()
        .trim()
        .to_owned();
    let cargo_version = cargo_version();

    assert_eq!(codex["name"], "guru-maker");
    assert_eq!(codex["version"], cargo_version);
    assert!(codex["interface"]["defaultPrompt"].is_array());
    assert_eq!(codex_marketplace["name"], "monarchjuno");
    assert_eq!(codex_marketplace["plugins"][0]["source"]["source"], "local");
    assert_eq!(codex_marketplace["plugins"][0]["source"]["path"], "./");
    assert!(codex_marketplace["plugins"][0].get("policy").is_none());
    assert_eq!(claude["name"], "guru-maker");
    assert_eq!(claude["version"], cargo_version);
    assert_eq!(marketplace["name"], "monarchjuno");
    assert_eq!(marketplace["plugins"][0]["source"], "./");
    assert_eq!(codex["version"], runtime_version);
    assert_eq!(claude["version"], runtime_version);

    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"guru-maker\""));
    assert_eq!(runtime_version, cargo_version);
    assert!(cargo.contains("name = \"gurumaker\""));
}

#[test]
fn public_skills_share_cross_host_runtime_delivery() {
    let posix = root().join("shared/gurumaker/scripts/gurumaker.sh");
    let windows = root().join("shared/gurumaker/scripts/gurumaker.ps1");
    let runtime_reference = root().join("shared/gurumaker/references/runtime.md");
    assert!(posix.is_file());
    assert!(windows.is_file());
    assert!(runtime_reference.is_file());
    assert!(!root().join("shared/gurumaker/SKILL.md").exists());

    let runtime = fs::read_to_string(runtime_reference).unwrap();
    assert!(runtime.contains("five public Skills"));
    assert!(runtime.contains("../../shared/gurumaker/scripts/gurumaker.sh"));
    assert!(runtime.contains("..\\..\\shared\\gurumaker\\scripts\\gurumaker.ps1"));

    for wrapper in [&posix, &windows] {
        let content = fs::read_to_string(wrapper).unwrap();
        assert!(content.contains("GURUMAKER_RUNTIME_BIN"));
        assert!(content.contains("GURUMAKER_RUNTIME_HOME"));
        assert!(content.contains("GURUMAKER_RELEASE_BASE_URL"));
        assert!(content.contains("SHA256SUMS"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(fs::metadata(posix).unwrap().permissions().mode() & 0o111, 0);
    }
}

#[test]
fn public_skill_surface_is_exactly_the_five_routed_entrypoints() {
    let expected = [
        "guru-decision",
        "guru-memory",
        "guru-packs",
        "guru-research",
        "guru-train",
    ]
    .map(str::to_owned);
    let skills_root = root().join("skills");
    let mut actual = fs::read_dir(&skills_root)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .path()
                .join("SKILL.md")
                .is_file()
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, expected);

    for name in expected {
        let path = skills_root.join(&name);
        let skill = fs::read_to_string(path.join("SKILL.md")).unwrap();
        let metadata = fs::read_to_string(path.join("agents/openai.yaml")).unwrap();
        assert!(skill.starts_with(&format!("---\nname: {name}\n")));
        assert!(skill.contains("../../shared/gurumaker/references/runtime.md"));
        assert!(skill.contains("relative to this loaded `SKILL.md`"));
        assert!(!skill.contains("[TODO:"));
        assert!(metadata.contains(&format!("Use ${name}")));
    }

    let research = fs::read_to_string(skills_root.join("guru-research/SKILL.md")).unwrap();
    assert!(research.contains("Evidence"));
    assert!(research.contains("Wiki"));
    assert!(research.contains("new, refreshed, or user-supplied"));
    assert!(research.contains("regardless of asset class or analytical method"));
    assert!(research.contains("user-supplied point-in-time external evidence or data"));
    assert!(research.contains("../../shared/gurumaker/references/guru-mode.md"));
    assert!(research.contains("../../shared/gurumaker/references/research-capabilities.md"));

    let memory = fs::read_to_string(skills_root.join("guru-memory/SKILL.md")).unwrap();
    assert!(memory.contains("Wiki, Lens, Evidence, and Method"));
    assert!(memory.contains("Initialize, recall, or maintain"));
    assert!(memory.contains("analysis of supplied point-in-time external evidence or data"));
    assert!(memory.contains("does not apply to workspace `init`"));
    assert!(memory.contains("../../shared/gurumaker/references/guru-mode.md"));

    let decision = fs::read_to_string(skills_root.join("guru-decision/SKILL.md")).unwrap();
    assert!(decision.contains("Execution Brief"));
    assert!(decision.contains("non-executing"));
    assert!(decision.contains("requested point-in-time"));
    assert!(decision.contains("ordinary project Markdown artifact"));
    assert!(decision.contains("inspecting any prior project memory"));
    assert!(decision.contains("targeted Runtime search followed by exact-ID reads"));
    assert!(decision.contains("do not enumerate or open"));
    assert!(decision.contains("only project-memory read interface"));
    assert!(decision.contains("even when a record path or exact ID is already visible"));
    assert!(decision.contains("not a recursive inventory of the workspace"));
    assert!(decision.contains("Every record under"));
    assert!(decision.contains("even if the prompt calls it supplied, current"));
    assert!(decision.contains("../../shared/gurumaker/references/guru-mode.md"));

    let execution_brief = normalized("shared/gurumaker/references/execution-brief.md");
    assert!(execution_brief.contains("outside `guru-maker/`"));
    assert!(execution_brief.contains("Guru Mode governs Guru memory"));
    assert!(execution_brief.contains("does not create a Decision"));

    let training_contract = normalized("shared/gurumaker/references/training.md");
    for boundary in [
        "model fine-tuning",
        "background learning",
        "automatic conversation capture",
        "scoring system",
    ] {
        assert!(
            training_contract.contains(boundary),
            "training contract misses {boundary}"
        );
    }

    let packs = normalized("skills/guru-packs/SKILL.md");
    assert!(packs.contains("data-only"));
    assert!(packs.contains("GitHub"));
    assert!(packs.contains("exactly one reusable kind"));
    assert!(packs.contains("at most one installed"));
    assert!(packs.contains("The CLI namespace is `pack`"));
    assert!(packs.contains("never changes local memory"));
    assert!(packs.contains("Guru Mode never makes a Pack"));

    let train = fs::read_to_string(skills_root.join("guru-train/SKILL.md")).unwrap();
    assert!(train.contains("../../shared/gurumaker/references/guru-mode.md"));
}

#[test]
fn retrieval_reviser_and_learning_boundaries_are_explicit() {
    let retrieval = normalized("shared/gurumaker/references/retrieval.md");
    for required in [
        "Direct recall",
        "adjacent transfer",
        "reformulation",
        "exact ID",
        "Do not enumerate the memory tree",
        "only project-memory read interface",
        "not a recursive inventory of the workspace",
        "even when a prompt calls it supplied, current, or Evidence",
        "visited IDs",
        "`updates`",
        "`contradicts`",
    ] {
        assert!(
            retrieval.contains(required),
            "retrieval contract misses {required}"
        );
    }

    let research = normalized("shared/gurumaker/references/research-framing.md");
    for required in [
        "claim scope, period, and currentness",
        "supplied by the user is external evidence for task routing",
        "fit-for-purpose provenance",
        "disconfirming evidence",
        "technical work",
        "quantitative work",
        "portfolio, risk, or relative-value work",
    ] {
        assert!(
            research.contains(required),
            "research contract misses {required}"
        );
    }

    let capabilities = normalized("shared/gurumaker/references/research-capabilities.md");
    for required in [
        "smallest useful set",
        "user-specified capability",
        "missing or inaccessible",
        "Do not install, connect, or configure",
        "Reuse adequate project evidence",
        "normalize material differences",
        "evidence, not authority",
        "reproducible",
        "Examples are illustrative, not routing rules",
    ] {
        assert!(
            capabilities.contains(required),
            "research capability contract misses {required}"
        );
    }

    let training = normalized("shared/gurumaker/references/training.md");
    for required in [
        "search for the reusable concept",
        "outcome luck",
        "counterexample",
        "limits",
        "Separate write authorization from memory eligibility",
        "presentation preference is ineligible on its own",
        "analytical rationale",
    ] {
        assert!(
            training.contains(required),
            "training contract misses {required}"
        );
    }
}

#[test]
fn public_examples_are_us_led_without_narrowing_market_scope() {
    let combined = [
        "README.md",
        "installation.md",
        "docs/architecture.md",
        "docs/knowledge-model.md",
        "docs/decision-memory.md",
        "shared/gurumaker/references/record-contract.md",
    ]
    .map(normalized)
    .join(" ");

    for required in [
        "familiar U.S. instruments",
        "ticker:NVDA",
        "market- and asset-neutral",
        "market-neutral",
        "market-specific",
    ] {
        assert!(
            combined.contains(required),
            "public market examples miss {required}"
        );
    }
    assert!(!combined.contains("ticker:005930.KS"));
}

#[test]
fn agent_e2e_matrix_is_diverse_isolated_and_qualitatively_reviewable() {
    let matrix = json("tests/e2e/cases.json");
    assert!(matrix.get("model").is_none());
    assert!(matrix.get("reasoning_effort").is_none());

    let cases = matrix["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    let ids = cases
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 30);

    for skill in [
        "guru-research",
        "guru-memory",
        "guru-decision",
        "guru-train",
        "guru-packs",
    ] {
        assert!(
            cases.iter().any(|case| case["skill"] == skill),
            "E2E matrix misses {skill}"
        );
    }
    assert!(
        cases
            .iter()
            .filter(|case| case["persona"] == "novice-installed-only")
            .count()
            >= 3
    );
    assert_eq!(matrix["qualitative_rubric"].as_array().unwrap().len(), 5);

    let harness = normalized("scripts/e2e-agent-eval.py");
    for required in [
        "--ephemeral",
        "--ignore-user-config",
        "model_reasoning_effort",
        "GURU_MAKER_E2E_MODEL",
        "tempfile.mkdtemp",
        "strict E2E isolation requires --jobs 1",
        "qualitative_review_status",
        "artifact_contents",
        "changed_content_regex",
    ] {
        assert!(
            harness.contains(required),
            "agent E2E harness misses {required}"
        );
    }
}

#[test]
fn record_and_pack_context_boundaries_are_explicit() {
    let records = normalized("shared/gurumaker/references/record-contract.md");
    for required in [
        "information horizon",
        "reproducible derived result",
        "`aliases`, `entities`, or `tags`",
        "source-transcription",
        "reusable for investment research, analysis, risk, or decision-making",
        "personal or presentation preference does not qualify",
        "`supports`",
        "`updates`",
        "`contradicts`",
    ] {
        assert!(
            records.contains(required),
            "record contract misses {required}"
        );
    }

    let packs = normalized("shared/gurumaker/references/workspace-contract.md");
    for required in [
        "One hidden source slot",
        "copies the declared elements once",
        "Never search",
        "preserves local memory",
    ] {
        assert!(packs.contains(required), "pack contract misses {required}");
    }
}

#[test]
fn guru_mode_is_default_auto_and_explicitly_bounded() {
    let mode = normalized("shared/gurumaker/references/guru-mode.md");
    for required in [
        "`auto` is the default",
        "minimum sufficient memory change",
        "Separate write authorization from memory eligibility",
        "presentation defaults",
        "In `off`",
        "explicitly asks",
        "not a background process",
        "not a filesystem permission or security boundary",
        "Guru Packs",
        "Git",
        "external execution",
    ] {
        assert!(
            mode.contains(required),
            "Guru Mode contract misses {required}"
        );
    }

    let combined = [
        "README.md",
        "installation.md",
        "docs/architecture.md",
        "docs/knowledge-model.md",
        "docs/decision-memory.md",
        "shared/gurumaker/references/workspace-contract.md",
        "shared/gurumaker/references/runtime.md",
    ]
    .map(normalized)
    .join(" ");
    for required in [
        ".guru-maker/mode",
        "gurumaker mode",
        "`auto`",
        "`off`",
        "explicit",
    ] {
        assert!(
            combined.contains(required),
            "documented Guru Mode misses {required}"
        );
    }
}

#[test]
fn singular_collections_and_kind_specific_packs_are_the_only_documented_model() {
    let combined = [
        "README.md",
        "installation.md",
        "docs/architecture.md",
        "docs/knowledge-model.md",
        "docs/decision-memory.md",
        "docs/community-plugins.md",
        "shared/gurumaker/references/workspace-contract.md",
        "shared/gurumaker/references/record-contract.md",
        ".github/workflows/ci.yml",
    ]
    .map(normalized)
    .join(" ");
    for required in [
        "guru-maker/wiki/",
        "guru-maker/lens/",
        "guru-maker/method/",
        "guru-maker/evidence/",
        "guru-maker/decision/",
        ".guru-maker/packs/",
        "guru-pack.json",
        "gurumaker pack",
        "nested subject",
    ] {
        assert!(combined.contains(required), "contract misses {required}");
    }
    for retired in [
        "guru-maker/wikis/",
        "guru-maker/lenses/",
        "guru-maker/methods/",
        "guru-maker/decisions/",
        ".guru-maker/plugins/",
        "guru-plugin.json",
        "gurumaker plugin",
    ] {
        assert!(!combined.contains(retired), "contract retains {retired}");
    }
}

#[test]
fn repository_development_skill_stays_outside_the_public_plugin_surface() {
    let skill = root().join(".agents/skills/guru-maker-development/SKILL.md");
    assert!(skill.is_file());
    let content = fs::read_to_string(skill).unwrap();
    assert!(content.starts_with("---\nname: guru-maker-development\n"));
    assert!(content.contains("scripts/verify.sh"));
    assert!(!content.contains("[TODO:"));
    assert!(root().join("scripts/verify.sh").is_file());
    assert!(root().join("scripts/test-env.sh").is_file());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in ["scripts/verify.sh", "scripts/test-env.sh"] {
            assert_ne!(
                fs::metadata(root().join(script))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111,
                0,
                "{script} must be executable"
            );
        }
    }
}

#[test]
fn plugin_hooks_are_minimal_shared_guardrails() {
    let hooks = json("hooks/hooks.json");
    let events = hooks["hooks"].as_object().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.contains_key("PreToolUse"));
    assert!(events.contains_key("PostToolUse"));
    assert!(!events.contains_key("SessionStart"));
    assert!(!events.contains_key("UserPromptSubmit"));
    assert!(!events.contains_key("Stop"));

    for event in ["PreToolUse", "PostToolUse"] {
        let group = &events[event][0];
        assert_eq!(group["matcher"], "Edit|Write|apply_patch");
        assert_eq!(group["hooks"][0]["type"], "command");
        assert!(group["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("run-hook.sh"));
        assert!(group["hooks"][0]["commandWindows"]
            .as_str()
            .unwrap()
            .contains("run-hook.ps1"));
    }

    assert!(root().join("hooks/run-hook.sh").is_file());
    assert!(root().join("hooks/run-hook.ps1").is_file());
    assert!(json(".codex-plugin/plugin.json").get("hooks").is_none());
    assert!(json(".claude-plugin/plugin.json").get("hooks").is_none());
}

#[test]
fn repository_contains_only_the_thin_product() {
    for retired in [
        "manage.py",
        "pyproject.toml",
        "frontend",
        "apps",
        "workspace_templates",
    ] {
        assert!(
            !root().join(retired).exists(),
            "retired runtime remains: {retired}"
        );
    }

    let skills = [
        "guru-research",
        "guru-memory",
        "guru-decision",
        "guru-train",
        "guru-packs",
    ]
    .into_iter()
    .map(|name| fs::read_to_string(root().join("skills").join(name).join("SKILL.md")).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
    for required in [
        "investment memory",
        "guru-maker/",
        "Guru Mode",
        "Guru Packs",
        "Execution Brief",
    ] {
        assert!(skills.contains(required), "skills miss {required}");
    }
    for forbidden in [
        "Head Manager",
        "Portfolio Manager",
        "Risk Manager",
        "auto-trading",
    ] {
        assert!(!skills.contains(forbidden), "skills retain {forbidden}");
    }
}
