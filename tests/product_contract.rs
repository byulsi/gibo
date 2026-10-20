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
fn release_publication_uses_the_versioned_changelog_entry() {
    let version = cargo_version();
    let changelog = fs::read_to_string(root().join("CHANGELOG.md")).unwrap();
    let heading = format!("## {version} - ");
    let matching_headings = changelog
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with(&heading))
        .collect::<Vec<_>>();
    assert_eq!(
        matching_headings.len(),
        1,
        "CHANGELOG.md must contain exactly one section for {version}"
    );
    let body = changelog
        .lines()
        .skip(matching_headings[0].0 + 1)
        .take_while(|line| !line.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        body.lines().any(|line| line.starts_with("- ")),
        "the {version} changelog section must contain public release notes"
    );

    let workflow = fs::read_to_string(root().join(".github/workflows/release.yml")).unwrap();
    assert!(workflow.contains("CHANGELOG.md"));
    assert!(workflow.contains("--notes-file"));
    assert!(workflow.contains("gh release edit"));
    assert!(workflow.contains("sha256sum \"$asset\""));
    assert!(workflow.contains("[.name, .digest]"));
    assert!(workflow.contains("test \"$asset_digest\" = \"$expected_digest\""));
    assert!(workflow.contains("diff -u \"$LOCAL_ASSETS\" \"$PUBLISHED_ASSETS\""));
    assert!(!workflow.contains("--generate-notes"));
    assert!(!workflow.contains("--clobber"));
}

#[test]
fn public_identity_and_runtime_version_are_synchronized() {
    let claude = json(".claude-plugin/plugin.json");
    let marketplace = json(".claude-plugin/marketplace.json");
    let runtime_version = fs::read_to_string(root().join("shared/gibo/runtime/version"))
        .unwrap()
        .trim()
        .to_owned();
    let cargo_version = cargo_version();

    assert_eq!(claude["name"], "gibo");
    assert_eq!(claude["repository"], "https://github.com/byulsi/gibo");
    assert_eq!(claude["version"], cargo_version);
    assert_eq!(claude["version"], runtime_version);
    assert_eq!(marketplace["name"], "byulsi");
    assert_eq!(marketplace["plugins"][0]["name"], "gibo");
    assert_eq!(marketplace["plugins"][0]["source"], "./");
    assert!(!root().join(".codex-plugin").exists());
    assert!(!root().join(".agents").exists());

    let cargo = fs::read_to_string(root().join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"gibo\""));
    assert_eq!(runtime_version, cargo_version);
    assert!(cargo.contains("name = \"gibo\""));
}

#[test]
fn public_skills_share_cross_host_runtime_delivery() {
    let posix = root().join("shared/gibo/scripts/gibo.sh");
    let windows = root().join("shared/gibo/scripts/gibo.ps1");
    let runtime_reference = root().join("shared/gibo/references/runtime.md");
    assert!(posix.is_file());
    assert!(windows.is_file());
    assert!(runtime_reference.is_file());
    assert!(!root().join("shared/gibo/SKILL.md").exists());

    let runtime = fs::read_to_string(runtime_reference).unwrap();
    assert!(runtime.contains("five public Skills"));
    assert!(runtime.contains("../../shared/gibo/scripts/gibo.sh"));
    assert!(runtime.contains("..\\..\\shared\\gibo\\scripts\\gibo.ps1"));

    for wrapper in [&posix, &windows] {
        let content = fs::read_to_string(wrapper).unwrap();
        assert!(content.contains("GIBO_RUNTIME_BIN"));
        assert!(content.contains("GIBO_RUNTIME_HOME"));
        assert!(content.contains("GIBO_RELEASE_BASE_URL"));
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
    let expected = ["decision", "memory", "packs", "research", "train"].map(str::to_owned);
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
        assert!(!path.join("agents").exists());
        assert!(skill.starts_with(&format!("---\nname: {name}\n")));
        assert!(skill.contains("../../shared/gibo/references/runtime.md"));
        assert!(skill.contains("relative to this loaded `SKILL.md`"));
        assert!(!skill.contains("[TODO:"));
    }

    let research = fs::read_to_string(skills_root.join("research/SKILL.md")).unwrap();
    assert!(research.contains("Evidence"));
    assert!(research.contains("Wiki"));
    assert!(research.contains(
        "material natural-language investment or financial research in an initialized Gibo project"
    ));
    assert!(research.contains("another research capability supplies the domain procedure"));
    assert!(research.contains("user does not mention Gibo or saving"));
    assert!(research.contains("supplied-source capture without refresh or analysis"));
    assert!(research.contains("wait for a separate save request"));
    assert!(research.contains("every material research task in an initialized project"));
    assert!(research.contains("Host agents and specialized capabilities own research strategy"));
    assert!(research.contains("do not act as an agent or tool\nrouter"));
    assert!(research.contains("Never filesystem-read a `gibo/` record"));
    assert!(research.contains("Record Mode `off` or a\nno-write instruction does not disable"));
    assert!(research.contains("bounded activation pass"));
    assert!(research.contains("authored reusable inputs"));
    assert!(research.contains("single most consequential\nreusable role"));
    assert!(research.contains("one allowed concept-gap query"));
    assert!(research.contains("../../shared/gibo/references/record-mode.md"));
    assert!(research.contains("../../shared/gibo/references/research-capabilities.md"));
    let research_capabilities = normalized("shared/gibo/references/research-capabilities.md");
    for required in [
        "not a research router",
        "project-memory retrieval",
        "mode-gated retention boundaries",
        "the host agent owns question framing, research strategy, tool selection",
        "evidence evaluation, normalization, synthesis",
    ] {
        assert!(
            research_capabilities.contains(required),
            "research capability boundary misses {required}"
        );
    }

    let memory = fs::read_to_string(skills_root.join("memory/SKILL.md")).unwrap();
    assert!(memory.contains("Wiki, Lens, Evidence, and Method"));
    assert!(memory.contains("Recall or maintain"));
    assert!(memory.contains("analysis of supplied point-in-time external evidence or data"));
    assert!(memory.contains("pre-write search does not apply to workspace"));
    assert!(memory.contains("repeating the\nunchanged query is not a reformulation"));
    assert!(memory.contains("Use automatically"));
    assert!(memory.contains("does not mention Gibo, a Skill, or saving"));
    assert!(memory.contains("require the user to name Gibo"));
    assert!(memory.contains("short follow-up that clearly refers to saved investment memory"));
    assert!(memory.contains("rendering the Decision Graph"));
    assert!(memory.contains("Never run `rg`, `grep`, `find`, `ls`, `cat`, `sed`"));
    assert!(memory.contains("a no-match is not evidence"));
    assert!(memory.contains("../../shared/gibo/references/record-mode.md"));

    let decision = fs::read_to_string(skills_root.join("decision/SKILL.md")).unwrap();
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
    assert!(decision.contains("Use automatically"));
    assert!(decision.contains("does not mention Gibo or a Skill"));
    assert!(decision.contains("every requested material judgment in an initialized project"));
    assert!(decision.contains("casual explanation or open-ended exploration"));
    assert!(decision.contains("bounded activation pass"));
    assert!(decision.contains("each unique reusable input in its authored `uses`"));
    assert!(decision.contains("only plausibly material `supports` Evidence for evaluation"));
    assert!(decision.contains("carry forward\nonly observations actually reused"));
    assert!(decision.contains("one allowed concept-gap query"));
    assert!(decision.contains("../../shared/gibo/references/record-mode.md"));
    assert!(
        decision.contains("route any write through the fitting Memory, Research, or Train Skill")
    );
    assert!(decision.contains("not write it through Decision"));

    let execution_brief = normalized("shared/gibo/references/execution-brief.md");
    assert!(execution_brief.contains("outside `gibo/`"));
    assert!(execution_brief.contains("Record Mode governs Gibo memory"));
    assert!(execution_brief.contains("does not create a Decision"));

    let training_contract = normalized("shared/gibo/references/training.md");
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
    assert!(training_contract.contains("One anecdote cannot establish a universal causal rule"));
    assert!(training_contract.contains("do not silently replace it with a safer counter-rule"));

    let packs = normalized("skills/packs/SKILL.md");
    assert!(packs.contains("data-only"));
    assert!(packs.contains("GitHub"));
    assert!(packs.contains("exactly one reusable kind"));
    assert!(packs.contains("at most one installed"));
    assert!(packs.contains("The CLI namespace is `pack`"));
    assert!(packs.contains("never changes local memory"));
    assert!(packs.contains("Record Mode never makes a Pack"));

    let train = fs::read_to_string(skills_root.join("train/SKILL.md")).unwrap();
    assert!(train.contains("../../shared/gibo/references/record-mode.md"));
    assert!(train.contains("never invoke this Skill implicitly"));
    assert!(train.contains("read that smallest supplied artifact"));
    assert!(packs.contains("never invoke Pack work implicitly"));

    for skill in ["memory", "research", "decision", "train"] {
        let content = fs::read_to_string(skills_root.join(skill).join("SKILL.md")).unwrap();
        assert!(
            content.contains("../../shared/gibo/references/curation.md"),
            "{skill} must load the shared curation contract before writing"
        );
    }
    for skill in ["memory", "research", "decision", "train", "packs"] {
        let content = fs::read_to_string(skills_root.join(skill).join("SKILL.md")).unwrap();
        assert!(
            content.contains("Never substitute a user-global `.codex` path"),
            "{skill} must keep shared references co-located with its loaded Skill"
        );
    }
}

#[test]
fn retrieval_reviser_and_learning_boundaries_are_explicit() {
    let retrieval = normalized("shared/gibo/references/retrieval.md");
    for required in [
        "Direct recall",
        "adjacent transfer",
        "reformulation",
        "exact ID",
        "compact-candidate query",
        "Repeated `--kind` values are an OR filter",
        "one compact concept query",
        "at most six candidate cards",
        "no record body",
        "lightweight subagent",
        "at least a 5 percentage-point",
        "Top-1 selection improvement",
        "knowledge neighbors",
        "Do not recurse",
        "Do not enumerate the memory tree",
        "only project-memory read interface",
        "not a recursive inventory of the workspace",
        "even when a prompt calls it supplied, current, or Evidence",
        "translate the same concept once into the likely",
        "Do not repeat the unchanged query",
        "visited IDs",
        "A non-forked named lineage stays within two lineage",
        "may add the separate one concept-gap query",
        "do not exact-read the just-written record",
        "`updates`",
        "`contradicts`",
        "every material research task or requested judgment",
        "one compact target query",
        "Activate selected memory for material work",
        "Candidate selection is discovery, not application",
        "each unique local Wiki, Lens, or Method in its authored `uses`",
        "From the Decision's reasoning and the initial candidate cards",
        "An evaluation read does not imply reuse",
        "carry forward only an observation the answer actually reuses",
        "one authored hop",
        "single most consequential reusable role",
        "exactly one compact concept query",
        "Do not query once per kind",
        "trace every consequential threshold",
        "An exact ID is the Runtime's internal authority and control address",
        "it is not syntax the user must know or type",
        "limits graph depth, duplicate reads, and search lanes",
        "do not sample them or partially rely on the Decision",
        "narrow fact",
    ] {
        assert!(
            retrieval.contains(required),
            "retrieval contract misses {required}"
        );
    }

    let research = normalized("shared/gibo/references/research-framing.md");
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

    let capabilities = normalized("shared/gibo/references/research-capabilities.md");
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

    let training = normalized("shared/gibo/references/training.md");
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
        "docs/guru-maker-readme.md",
        "docs/guru-maker-installation.md",
        "docs/architecture.md",
        "docs/knowledge-model.md",
        "docs/decision-memory.md",
        "shared/gibo/references/record-contract.md",
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
fn record_and_pack_context_boundaries_are_explicit() {
    let records = normalized("shared/gibo/references/record-contract.md");
    for required in [
        "information horizon",
        "reproducible derived result",
        "`aliases`, `entities`, or `tags`",
        "central tag registry",
        "`see_also`",
        "Wiki-only",
        "Obsidian-style `[[filename]]` links are not canonical",
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

    let packs = normalized("shared/gibo/references/workspace-contract.md");
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
fn wiki_authoring_is_llm_friendly_and_linked_without_becoming_a_graph() {
    let records = normalized("shared/gibo/references/record-contract.md");
    for required in [
        "science, technology, industry",
        "one coherent concept or question",
        "stable semantic ID",
        "shallow subject folders",
        "descriptive, unique ATX headings",
        "remain intelligible when retrieved alone",
        "general encyclopedia dump",
        "experimental result, benchmark result",
        "time-sensitive state in Evidence with provenance and a cutoff",
        "`see_also` is optional and Wiki-only",
        "Runtime derives incoming backlinks",
        "does not create a Decision Graph edge",
        "A Wiki Pack must include every exact `see_also` target",
        "never follows or automatically adds a target",
        "never triggers Runtime or background copying, loading, or recursive traversal",
        "explicit material-work activation pass is the one exception",
    ] {
        assert!(
            records.contains(required),
            "Wiki authoring contract misses {required}"
        );
    }

    let knowledge_model = normalized("docs/knowledge-model.md");
    for required in [
        "science, technology, industry",
        "not a general encyclopedia",
        "stable semantic ID",
        "lexical retrieval section boundaries",
        "avoid line-start ATX-like comment lines inside fenced examples",
        "provenance and a cutoff",
        "incoming backlinks",
        "does not affect search score",
        "appear in the Decision Graph",
        "Wiki Pack must declare every `see_also` target",
        "Obsidian-style `[[filename]]` links are not canonical",
    ] {
        assert!(
            knowledge_model.contains(required),
            "knowledge model misses Wiki boundary {required}"
        );
    }

    let readme = normalized("docs/guru-maker-readme.md");
    for required in [
        "Reusable scientific and technical knowledge",
        "company or industry research",
        "provenance-bound Evidence",
    ] {
        assert!(readme.contains(required), "README misses {required}");
    }

    let installation = normalized("docs/guru-maker-installation.md");
    for required in [
        "memory contract is domain-neutral",
        "without naming a security",
        "does not collect general encyclopedia material",
        "docs/knowledge-model.md#wiki-authoring",
    ] {
        assert!(
            installation.contains(required),
            "installation guide misses {required}"
        );
    }

    let architecture = normalized("docs/architecture.md");
    for required in [
        "stable semantic ID owns identity",
        "Shallow subject folders are optional organization",
        "independently intelligible retrieval sections",
        "automatic context expansion",
        "derived backlinks",
        "navigation, not analytical relationships",
    ] {
        assert!(
            architecture.contains(required),
            "architecture misses {required}"
        );
    }
}

#[test]
fn linked_wiki_is_current_while_semantic_retrieval_remains_measured() {
    let readme = normalized("docs/guru-maker-readme.md");
    for required in [
        "Wiki records may name a few related local Wiki IDs with `see_also`",
        "derives backlinks on demand",
        "does not infer links",
        "automatically load neighbor content",
        "not recommended default behavior",
        "at least five percentage points",
        "evaluation gates, not promised features",
        "current retrieval remains project-scoped lexical candidate search followed by exact-ID reads",
        "rebuildable from local Markdown",
        "ignore Pack source checkouts",
        "Embeddings or a vector store require an explicit revision",
        "without unacceptable precision, context-token, latency, or migration cost",
    ] {
        assert!(readme.contains(required), "roadmap misses {required}");
    }
}

#[test]
fn retrieval_scaling_is_bounded_measured_and_keeps_markdown_canonical() {
    let scaling = normalized("docs/retrieval-scaling.md");
    for required in [
        "Search performance, retrieval quality, memory activation, and curation are separate concerns",
        "stable internal authority and control address",
        "linear worst case in the Markdown scanned",
        "result-order parity for narrow, broad common-term, candidate, repeated-kind, and entity-filter searches",
        "literal retrieval invariants before timing",
        "exact-ID Top-1",
        "no-answer precision",
        "declared-input coverage",
        "activation false negatives and false positives",
        "Corpus growth never authorizes lossy compression",
        "authored section is the retrieval and context unit",
        "raw-section fallback",
        "Latency or peak-memory failure",
        "disposable fielded lexical index",
        "fail open to the correct direct scan",
        "Semantic recall failure",
        "fixed context-token budget",
        "complete model and index lifecycle",
        "Global synthesis failure",
    ] {
        assert!(scaling.contains(required), "scaling contract misses {required}");
    }

    let benchmark = normalized("scripts/retrieval-benchmark.py");
    for required in [
        "BASELINE_REF = \"v0.0.3\"",
        "default=[100, 1000, 5000]",
        "default=20",
        "runtime_source_sha256",
        "benchmark_sha256",
        "warm_p50",
        "warm_p95",
        "candidate_over_v0.0.3_p50",
        "signed_number_top1",
        "wrong_sign_no_match",
        "repeated_kind_candidate_top1",
        "broad_candidate_search",
        "entity_filter_search",
        "alternating AB/BA within each operation",
        "output_parity",
    ] {
        assert!(
            benchmark.contains(required),
            "retrieval benchmark misses {required}"
        );
    }
    assert!(!benchmark.contains("cytoscape"));
    assert!(!benchmark.contains("node_modules"));
}

#[test]
fn record_mode_is_default_auto_and_explicitly_bounded() {
    let mode = normalized("shared/gibo/references/record-mode.md");
    for required in [
        "`auto` is the default",
        "minimum sufficient memory change",
        "no more than five existing local",
        "Run `knowledge health`",
        "must not change IDs",
        "Separate write authorization from memory eligibility",
        "presentation defaults",
        "In `off`",
        "explicitly asks",
        "not a background process",
        "not a filesystem permission or security boundary",
        "Gibo Packs",
        "Git",
        "external execution",
        "never names Gibo",
        "does not need to mention Gibo",
        "already initialized project",
    ] {
        assert!(
            mode.contains(required),
            "Record Mode contract misses {required}"
        );
    }

    let combined = [
        "README.md",
        "installation.md",
        "docs/architecture.md",
        "docs/knowledge-model.md",
        "docs/decision-memory.md",
        "shared/gibo/references/workspace-contract.md",
        "shared/gibo/references/runtime.md",
    ]
    .map(normalized)
    .join(" ");
    for required in [".gibo/mode", "gibo mode", "`auto`", "`off`", "explicit"] {
        assert!(
            combined.contains(required),
            "documented Record Mode misses {required}"
        );
    }
}

#[test]
fn in_task_curation_is_kind_specific_bounded_and_host_owned() {
    let curation = normalized("shared/gibo/references/curation.md");
    for required in [
        "host-owned semantic work",
        "not a background process",
        "compact candidates",
        "no more than five existing records",
        "knowledge health --kind",
        "one knowledge check",
        "path returned on its candidate card",
        "standalone Runtime invocation",
        "**Wiki** is a connected library",
        "**Lens** is a small library",
        "**Method** is a catalog",
        "**Evidence** is append-oriented",
        "author the exact `updates` or `contradicts` relationship",
        "**Decision** preserves judgment history",
        "at least four elements",
        "no more than two subject levels",
        "central tag registry",
        "separate keywords field",
        "must not change IDs",
        "delete or merge records",
        "installed Pack source checkouts",
        "first show a plan and the affected IDs",
        "scheduled cleanup",
        "Classify the durable payload",
        "Stable explanatory fact",
        "Source- or calculation-bound observation",
        "Prefer one primary memory improvement",
        "must not generalize from one observation alone",
        "Casual explanation or open-ended exploration",
        "entity match alone does not authorize unrelated cleanup",
        "Rejecting an ineligible requested memory does not authorize",
        "before any workspace inventory or Gibo command",
        "do not run another search, list, or exact read",
    ] {
        assert!(
            curation.contains(required),
            "curation contract misses {required}"
        );
    }

    let mode = normalized("shared/gibo/references/record-mode.md");
    assert!(
        mode.contains("An explicit write authorizes the requested record, not adjacent cleanup")
    );
    assert!(mode.contains(
        "the five-record cap applies to incidental Auto curation, not to an explicitly reviewed scope"
    ));

    let architecture = normalized("docs/architecture.md");
    assert!(architecture.contains(
        "common Markdown envelope but intentionally have different structures and lifecycles"
    ));
    assert!(architecture.contains("no more than five existing local records"));
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
        "shared/gibo/references/workspace-contract.md",
        "shared/gibo/references/record-contract.md",
        ".github/workflows/ci.yml",
    ]
    .map(normalized)
    .join(" ");
    for required in [
        "gibo/wiki/",
        "gibo/lens/",
        "gibo/method/",
        "gibo/evidence/",
        "gibo/decision/",
        ".gibo/packs/",
        "gibo-pack.json",
        "gibo pack",
        "nested subject",
    ] {
        assert!(combined.contains(required), "contract misses {required}");
    }
    for retired in [
        "gibo/wikis/",
        "gibo/lenses/",
        "gibo/methods/",
        "gibo/decisions/",
        ".gibo/plugins/",
        "guru-plugin.json",
        "gibo plugin",
    ] {
        assert!(!combined.contains(retired), "contract retains {retired}");
    }
}

#[test]
fn maintainer_scripts_are_present_and_executable() {
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

    let skills = ["research", "memory", "decision", "train", "packs"]
        .into_iter()
        .map(|name| fs::read_to_string(root().join("skills").join(name).join("SKILL.md")).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "investment memory",
        "gibo/",
        "Record Mode",
        "Gibo Packs",
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

#[test]
fn data_connections_are_pinned_read_only_and_keep_secrets_out() {
    let mcp = json(".mcp.json");
    let servers = mcp["mcpServers"].as_object().unwrap();
    let mut names = servers.keys().cloned().collect::<Vec<_>>();
    names.sort();
    assert_eq!(names, ["dart", "krx", "openbb"]);

    for (name, server) in servers {
        assert_eq!(server["command"], "sh", "{name} must start through sh");
        let launcher = server["args"][0].as_str().unwrap();
        let relative = launcher
            .strip_prefix("${CLAUDE_PLUGIN_ROOT}/")
            .unwrap_or_else(|| panic!("{name} launcher must live in the plugin"));
        assert!(root().join(relative).is_file(), "{name} launcher missing");
        for (key, value) in server
            .get("env")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let value = value.as_str().unwrap();
            assert!(
                value.starts_with("${user_config.") && value.ends_with('}'),
                "{name} {key} must come from userConfig, not a literal"
            );
        }
    }

    let plugin = json(".claude-plugin/plugin.json");
    let options = plugin["userConfig"].as_object().unwrap();
    assert_eq!(options["dart_api_key"]["sensitive"], true);
    assert_eq!(options["krx_password"]["sensitive"], true);

    let setup = normalized("mcp/setup.sh");
    for pinned in [
        "openbb-mcp-server==2.0.1",
        "openbb-core[pandas]==2.0.1",
        "openbb-yfinance==2.0.0",
        "korean-dart-mcp@0.10.1",
        "--omit=optional",
        "pykrx==1.2.9",
        "pykrx-mcp@47b92e547f703186b7f9e8c938599ef92fe71cc3",
        "\"mcp<2\"",
    ] {
        assert!(setup.contains(pinned), "setup.sh must pin {pinned}");
    }
    for name in ["openbb", "dart", "krx"] {
        let launcher = normalized(&format!("mcp/{name}.sh"));
        assert!(
            launcher.contains(&format!("relay.py\" --name {name}")),
            "{name} must run through the Gibo relay"
        );
    }
    assert!(normalized("mcp/openbb.sh").contains("--lazy"));
    for helper in ["relay_dart.py", "relay_krx.py", "dart-markdown.mjs"] {
        assert!(
            root().join("mcp").join(helper).is_file(),
            "{helper} missing"
        );
    }
}
