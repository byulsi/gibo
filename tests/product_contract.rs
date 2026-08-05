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

    let development_skill =
        fs::read_to_string(root().join(".agents/skills/guru-maker-development/SKILL.md")).unwrap();
    assert!(development_skill.contains("public GitHub Release body"));
    assert!(development_skill.contains("published body and expected assets"));
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
    assert!(research.contains(
        "material natural-language investment or financial research in an initialized Guru Maker project"
    ));
    assert!(research.contains("another research capability supplies the domain procedure"));
    assert!(research.contains("user does not mention Guru Maker or saving"));
    assert!(research.contains("supplied-source capture without refresh or analysis"));
    assert!(research.contains("wait for a separate save request"));
    assert!(research.contains("every material research task in an initialized project"));
    assert!(research.contains("Host agents and specialized capabilities own research strategy"));
    assert!(research.contains("do not act as an agent or tool\nrouter"));
    assert!(research.contains("Never filesystem-read a `guru-maker/` record"));
    assert!(research.contains("Guru Mode `off` or a\nno-write instruction does not disable"));
    assert!(research.contains("bounded activation pass"));
    assert!(research.contains("authored reusable inputs"));
    assert!(research.contains("single most consequential\nreusable role"));
    assert!(research.contains("one allowed concept-gap query"));
    assert!(research.contains("../../shared/gurumaker/references/guru-mode.md"));
    assert!(research.contains("../../shared/gurumaker/references/research-capabilities.md"));
    let research_capabilities = normalized("shared/gurumaker/references/research-capabilities.md");
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

    let memory = fs::read_to_string(skills_root.join("guru-memory/SKILL.md")).unwrap();
    assert!(memory.contains("Wiki, Lens, Evidence, and Method"));
    assert!(memory.contains("Recall or maintain"));
    assert!(memory.contains("analysis of supplied point-in-time external evidence or data"));
    assert!(memory.contains("pre-write search does not apply to workspace"));
    assert!(memory.contains("repeating the\nunchanged query is not a reformulation"));
    assert!(memory.contains("Use automatically"));
    assert!(memory.contains("does not mention Guru Maker, a Skill, or saving"));
    assert!(memory.contains("require the user to name Guru Maker"));
    assert!(memory.contains("short follow-up that clearly refers to saved investment memory"));
    assert!(memory.contains("rendering the Decision Graph"));
    assert!(memory.contains("Never run `rg`, `grep`, `find`, `ls`, `cat`, `sed`"));
    assert!(memory.contains("a no-match is not evidence"));
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
    assert!(decision.contains("Use automatically"));
    assert!(decision.contains("does not mention Guru Maker or a Skill"));
    assert!(decision.contains("every requested material judgment in an initialized project"));
    assert!(decision.contains("casual explanation or open-ended exploration"));
    assert!(decision.contains("bounded activation pass"));
    assert!(decision.contains("each unique reusable input in its authored `uses`"));
    assert!(decision.contains("only plausibly material `supports` Evidence for evaluation"));
    assert!(decision.contains("carry forward\nonly observations actually reused"));
    assert!(decision.contains("one allowed concept-gap query"));
    assert!(decision.contains("../../shared/gurumaker/references/guru-mode.md"));
    assert!(
        decision.contains("route any write through the fitting Memory, Research, or Train Skill")
    );
    assert!(decision.contains("not write it through Decision"));

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
    assert!(training_contract.contains("One anecdote cannot establish a universal causal rule"));
    assert!(training_contract.contains("do not silently replace it with a safer counter-rule"));

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
    assert!(train.contains("never invoke this Skill implicitly"));
    assert!(train.contains("read that smallest supplied artifact"));
    assert!(packs.contains("never invoke Pack work implicitly"));

    for skill in [
        "guru-memory",
        "guru-research",
        "guru-decision",
        "guru-train",
    ] {
        let content = fs::read_to_string(skills_root.join(skill).join("SKILL.md")).unwrap();
        assert!(
            content.contains("../../shared/gurumaker/references/curation.md"),
            "{skill} must load the shared curation contract before writing"
        );
    }
    for skill in [
        "guru-memory",
        "guru-research",
        "guru-decision",
        "guru-train",
        "guru-packs",
    ] {
        let content = fs::read_to_string(skills_root.join(skill).join("SKILL.md")).unwrap();
        assert!(
            content.contains("Never substitute a user-global `.codex` path"),
            "{skill} must keep shared references co-located with its loaded Skill"
        );
    }
}

#[test]
fn retrieval_reviser_and_learning_boundaries_are_explicit() {
    let retrieval = normalized("shared/gurumaker/references/retrieval.md");
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
    assert!(
        cases.len() >= 69,
        "E2E matrix must keep at least 69 realistic prompts"
    );
    let ids = cases
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), cases.len());

    assert!(
        cases
            .iter()
            .filter(|case| case["prompt"]
                .as_str()
                .is_some_and(|prompt| prompt.chars().any(|ch| ('가'..='힣').contains(&ch))))
            .count()
            >= 10,
        "E2E matrix needs substantial Korean coverage"
    );
    assert!(
        cases
            .iter()
            .filter(|case| case["prompt"]
                .as_str()
                .is_some_and(|prompt| !prompt.to_ascii_lowercase().contains("guru")))
            .count()
            >= 20,
        "E2E matrix needs natural requests without Guru invocation"
    );

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
    let linked_wiki = cases.iter().find(|case| case["id"] == "M31").unwrap();
    assert!(linked_wiki["prompt"].as_str().unwrap().contains("see_also"));
    assert!(linked_wiki["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["type"] == "wiki_one_way_link"));
    assert!(linked_wiki["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["pattern"]
            .as_str()
            .is_some_and(|pattern| pattern.contains("knowledge health"))));
    for case_id in ["M32", "M33", "M34", "P35"] {
        assert!(
            cases.iter().any(|case| case["id"] == case_id),
            "E2E matrix misses {case_id}"
        );
    }
    let auto_budget = cases.iter().find(|case| case["id"] == "M32").unwrap();
    assert!(!auto_budget["prompt"].as_str().unwrap().contains("see_also"));
    assert!(auto_budget["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["type"] == "changed_at_most_matching" && check["count"] == 5 }));
    assert!(auto_budget["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["record_id"] == "wiki:logical-qubits"));
    let lens_dedup = cases.iter().find(|case| case["id"] == "M33").unwrap();
    assert!(lens_dedup["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["type"] == "glob_exact" && check["count"] == 1));
    let evidence_lineage = cases.iter().find(|case| case["id"] == "M34").unwrap();
    assert!(evidence_lineage["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["pattern"]
            .as_str()
            .is_some_and(|pattern| pattern.contains("contradicts"))));
    let pack_closure = cases.iter().find(|case| case["id"] == "P35").unwrap();
    assert!(pack_closure["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["type"] == "path_absent" && check["path"] == "linked-pack"));

    let natural_research = cases.iter().find(|case| case["id"] == "I57").unwrap();
    let natural_research_prompt = natural_research["prompt"].as_str().unwrap().to_lowercase();
    for cue in ["guru", "skill", "save", "remember", "retain", "record"] {
        assert!(
            !natural_research_prompt.contains(cue),
            "I57 leaks explicit cue {cue}"
        );
    }
    assert!(natural_research["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["type"] == "changed_exactly_one_matching"
            && check["pattern"]
                .as_str()
                .is_some_and(|pattern| pattern.contains("evidence"))));

    let natural_decision = cases.iter().find(|case| case["id"] == "I58").unwrap();
    let natural_decision_prompt = natural_decision["prompt"].as_str().unwrap().to_lowercase();
    for cue in ["guru", "skill", "저장", "기억", "기록"] {
        assert!(
            !natural_decision_prompt.contains(cue),
            "I58 leaks explicit cue {cue}"
        );
    }
    assert!(natural_decision["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| check["type"] == "changed_exactly_one_matching"
            && check["pattern"]
                .as_str()
                .is_some_and(|pattern| pattern.contains("decision"))));

    let input_activation = cases.iter().find(|case| case["id"] == "D68").unwrap();
    let input_prompt = input_activation["prompt"].as_str().unwrap().to_lowercase();
    for cue in ["guru", "skill", "memory", "save", "record"] {
        assert!(!input_prompt.contains(cue), "D68 leaks explicit cue {cue}");
    }
    let input_checks = input_activation["checks"].as_array().unwrap();
    for (record_id, expected_count) in [
        ("decision:atlas-utilization-prior", 1_u64),
        ("lens:persistence-confirmation-gate", 1_u64),
        ("evidence:atlas-utilization-current", 1_u64),
    ] {
        assert!(input_checks.iter().any(|check| {
            check["type"] == "runtime_operation_count"
                && check["operation"]["action"] == "read"
                && check["operation"]["value"] == record_id
                && check["count"] == expected_count
        }));
    }
    assert!(input_checks
        .iter()
        .any(|check| check["type"] == "memory_reads_via_runtime"));
    assert!(input_checks.iter().any(|check| {
        check["type"] == "changed_frontmatter_contains"
            && check["field"] == "uses"
            && check["value"] == "lens:persistence-confirmation-gate"
    }));

    let gap_activation = cases.iter().find(|case| case["id"] == "D69").unwrap();
    let gap_prompt = gap_activation["prompt"].as_str().unwrap().to_lowercase();
    for cue in ["guru", "skill", "memory", "save", "record", "lens"] {
        assert!(!gap_prompt.contains(cue), "D69 leaks explicit cue {cue}");
    }
    let gap_checks = gap_activation["checks"].as_array().unwrap();
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "runtime_operation_count"
            && check["operation"]["action"] == "search"
            && check["count"] == 3
    }));
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "runtime_operation_count"
            && check["operation"]["action"] == "search"
            && check["operation"]["kinds"] == serde_json::json!(["lens"])
            && check["operation"]["candidates"] == true
            && check["operation"]["limit"] == "6"
            && check["count"] == 1
    }));
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "runtime_operation_count"
            && check["operation"]["action"] == "search"
            && check["operation"]["kinds"] == serde_json::json!(["decision"])
            && check["operation"]["candidates"] == true
            && check["operation"]["limit"] == "6"
            && check["count"] == 1
    }));
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "runtime_operation_sequence"
            && check["operations"].as_array().is_some_and(|operations| {
                operations.len() == 2
                    && operations[0]["action"] == "search"
                    && operations[0]["kind"].is_null()
                    && operations[1]["action"] == "search"
                    && operations[1]["kinds"] == serde_json::json!(["decision"])
            })
    }));
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "runtime_operation_count"
            && check["operation"]["action"] == "read"
            && check["operation"]["value"] == "lens:look-through-concentration"
            && check["count"] == 1
    }));
    assert!(gap_checks.iter().any(|check| {
        check["type"] == "changed_frontmatter_contains"
            && check["field"] == "supports"
            && check["value"] == "evidence:dual-rally-current"
    }));

    for (case_id, kind, operation) in [
        ("I59", "wiki", "use"),
        ("I60", "method", "update"),
        ("I61", "evidence", "append"),
        ("I62", "decision", "append"),
        ("I63", "wiki", "update"),
        ("I64", "lens", "update"),
        ("I67", "wiki", "korean-use"),
    ] {
        let case = cases.iter().find(|case| case["id"] == case_id).unwrap();
        assert_eq!(case["fixture"], "large_corpus");
        assert!(
            case["checks"].as_array().unwrap().iter().any(|check| {
                check["glob"]
                    .as_str()
                    .is_some_and(|glob| glob.contains(&format!("guru-maker/{kind}")))
                    || check["pattern"]
                        .as_str()
                        .is_some_and(|pattern| pattern.contains(&format!("guru-maker/{kind}")))
                    || check["path"]
                        .as_str()
                        .is_some_and(|path| path.contains(&format!("guru-maker/{kind}")))
            }),
            "large-corpus {kind} {operation} case lacks a kind-specific assertion"
        );
        let checks = case["checks"].as_array().unwrap();
        assert!(
            checks
                .iter()
                .any(|check| check["type"] == "event_command_count"
                    && check["pattern"] == "knowledge read"
                    && check["max"].as_u64().is_some_and(|max| max <= 3)),
            "large-corpus {kind} {operation} case lacks a bounded exact-read assertion"
        );
    }

    for (case_id, kind) in [
        ("I40", "wiki"),
        ("I55", "method"),
        ("I56", "lens"),
        ("I57", "evidence"),
        ("I58", "decision"),
    ] {
        let case = cases.iter().find(|case| case["id"] == case_id).unwrap();
        let checks = case["checks"].as_array().unwrap();
        assert!(
            checks.iter().any(|check| {
                check["type"] == "changed_exactly_one_matching"
                    && check["pattern"]
                        .as_str()
                        .is_some_and(|pattern| pattern.contains(kind))
            }),
            "{case_id} must prove exactly one {kind} write"
        );
        assert!(
            checks.iter().any(|check| {
                check["type"] == "event_regex"
                    && check["pattern"].as_str().is_some_and(|pattern| {
                        pattern.contains("knowledge search")
                            && pattern.contains("--candidates")
                            && pattern.contains("--limit 6")
                    })
            }),
            "{case_id} must prove bounded pre-write candidate search"
        );
        assert!(checks.iter().any(|check| {
            check["type"] == "event_regex"
                && check["pattern"]
                    .as_str()
                    .is_some_and(|pattern| pattern.contains("knowledge health"))
        }));
        assert!(checks.iter().any(|check| {
            check["type"] == "event_regex"
                && check["pattern"]
                    .as_str()
                    .is_some_and(|pattern| pattern.contains("knowledge check"))
        }));
    }

    let harness = normalized("scripts/e2e-agent-eval.py");
    for required in [
        "--ephemeral",
        "--ignore-user-config",
        "model_reasoning_effort",
        "GURU_MAKER_E2E_MODEL",
        "tempfile.mkdtemp",
        "isolated_agent_home",
        "isolated_codex_home",
        "\"XDG_CONFIG_HOME\"",
        "disabled_user_skill_config",
        "skills.config=[",
        "external_guru_reads",
        "project_local_skill_isolation",
        "snapshot_agent_assets",
        "project_local_contract_integrity",
        "strict E2E isolation requires --jobs 1",
        "qualitative_review_status",
        "artifact_contents",
        "changed_content_regex",
        "changed_content_not_regex",
        "changed_at_most_matching",
        "event_command_count",
        "runtime_operation_count",
        "runtime_operation_sequence",
        "memory_reads_via_runtime",
        "extract_successful_commands",
        "runtime_operations(successful_commands)",
        "punctuation_chars=\";&|\\n\"",
        "memory_reads_via_runtime(commands, workspace)",
        "changed_frontmatter_contains",
        "record_frontmatter_list_equals",
        "wiki_one_way_link",
        "glob_exact",
        "glob_content_match_count",
        "validate_configuration",
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
fn wiki_authoring_is_llm_friendly_and_linked_without_becoming_a_graph() {
    let records = normalized("shared/gurumaker/references/record-contract.md");
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

    let readme = normalized("README.md");
    for required in [
        "Reusable scientific and technical knowledge",
        "company or industry research",
        "provenance-bound Evidence",
    ] {
        assert!(readme.contains(required), "README misses {required}");
    }

    let installation = normalized("installation.md");
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
    let readme = normalized("README.md");
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
fn guru_mode_is_default_auto_and_explicitly_bounded() {
    let mode = normalized("shared/gurumaker/references/guru-mode.md");
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
        "Guru Packs",
        "Git",
        "external execution",
        "never names Guru Maker",
        "does not need to mention Guru Maker",
        "already initialized project",
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
fn in_task_curation_is_kind_specific_bounded_and_host_owned() {
    let curation = normalized("shared/gurumaker/references/curation.md");
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
        "before any workspace inventory or Guru Maker command",
        "do not run another search, list, or exact read",
    ] {
        assert!(
            curation.contains(required),
            "curation contract misses {required}"
        );
    }

    let mode = normalized("shared/gurumaker/references/guru-mode.md");
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
