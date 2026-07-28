# Development and Validation

Guru Maker targets stable Rust and ships one native `gurumaker` binary.

```bash
scripts/verify.sh
```

`scripts/verify.sh` is the maintainer entrypoint. It runs formatting, Clippy,
the locked Rust test suite, a release build, portable metadata and shell
checks, and `scripts/test-env.sh`.

## Isolated test environment

`scripts/test-env.sh` creates a fresh temporary directory outside the checkout.
It builds or reuses the local debug Runtime, serves it through a local
file-based release fixture, verifies checksum-gated installation, initializes
two independent workspaces, and checks that Guru Mode defaults to `auto` and
remains project-scoped. The harness sets its own
`GURUMAKER_RUNTIME_HOME` and clears external Runtime overrides, so it cannot
read or modify a developer's normal Runtime directory, Guru Maker projects, or
Git credentials.

Run it directly during wrapper or workspace work:

```bash
scripts/test-env.sh
```

Pass `--keep` only when debugging a failure; the script prints the temporary
fixture path and leaves it for manual inspection. The default always removes
only the directory it created with `mktemp`.

## Model-backed agent E2E

The opt-in agent evaluation uses the 30-case matrix in `tests/e2e/cases.json`
with a maintainer-selected Codex model. It is intentionally separate from
`scripts/verify.sh` because it is slower, stochastic, and requires model
access. The public matrix defines behavior and failure criteria, not a
preferred model.

Validate fixture construction without an agent call:

```bash
python3 scripts/e2e-agent-eval.py --prepare-only
```

Run the strict public matrix with a model available in your Codex environment:

```bash
python3 scripts/e2e-agent-eval.py \
  --model YOUR_CODEX_MODEL \
  --reasoning-effort high \
  --jobs 1
```

You may instead set `GURU_MAKER_E2E_MODEL` and, optionally,
`GURU_MAKER_E2E_REASONING_EFFORT`. Keep unpublished holdouts and reviewed
result artifacts outside the repository so they remain independent and do not
turn a stochastic baseline into a product contract.

Strict isolation requires sequential execution. Each agent receives a fresh
temporary workspace outside the checkout and a copied Runtime binary. The
workspace is removed before the next case starts; final text, commands, changed
paths, and small text artifacts are retained in process memory and written to
an external results directory only after all agents finish. Keep expected
behavior and failure checks outside the agent workspace.

Review the deterministic result together with the five qualitative dimensions
in the matrix. A passing regex or file check does not excuse incorrect
analysis, hidden execution, cross-project access, over-retention, or a
case-specific rote workflow.

## Agent-oriented repository structure

`AGENTS.md` is the primary contributor contract. The public plugin exposes
only `skills/guru-{research,memory,decision,train,packs}/`. The separate
`.agents/skills/guru-maker-development/` Skill is for agents maintaining this
repository; it is excluded from the public plugin and must never be moved into
`skills/`.

Validate `.codex-plugin/plugin.json` with the Codex plugin validator, each
public `skills/guru-{research,memory,decision,train,packs}/` directory with the
skill validator, and the repository with:

```bash
claude plugin validate .
```

Use a temporary home/config for installation smokes, keeping the maintainer's
Codex and Claude Code configuration untouched.

Behavioral validation uses fresh temporary projects:

1. initialize two projects and install different kind-specific Guru Pack
   source slots;
2. confirm Guru Mode defaults to `auto`, an explicit `off` persists, and the
   setting remains isolated by project;
3. confirm each project's catalog and graph state remain isolated;
4. validate minimal frontmatter, timezone-aware `as_of`, Evidence provenance,
   duplicate IDs, and the minimal relationship matrix;
5. confirm direct contextual search finds financial identifiers and general
   Wiki knowledge;
6. confirm a narrow factual question remains conversational;
7. confirm a decision-grade request can create a Decision in `auto` and remains
   conversational in `off` without an explicit record request;
8. confirm a user-invoked training session produces only reviewed reusable
   Lens, Wiki, or Method diffs;
9. confirm a durable correction changes only local memory;
10. confirm the installed Pack source slot stays clean and is never loaded as
   project memory, while its copied local elements remain searchable;
11. confirm native edits to installed Pack source files are blocked and
   invalid local memory edits return corrective hook context;
12. confirm unrelated edits and edits outside initialized projects do not
    invoke knowledge validation;
13. confirm explicit action intent produces an Execution Brief while order
   state remains with the external capability;
14. run equivalent fresh-session smokes in Codex and Claude Code, including
   hook discovery and trust.

Release builds cover macOS arm64 and x86-64, Linux arm64 and x86-64, and
Windows x86-64. Validate the downloaded binary on each target before attaching
it to a release.

Runtime delivery changes also require:

```bash
sh -n shared/gurumaker/scripts/gurumaker.sh
GURUMAKER_RUNTIME_BIN="$PWD/target/debug/gurumaker" \
  sh shared/gurumaker/scripts/gurumaker.sh --runtime-status
```

Run the equivalent `gurumaker.ps1` smoke on Windows. CI runs the same isolated
release-fixture install, plus native wrapper and hook-launcher checks.

Before creating a release, update the same exact version in:

- `Cargo.toml`;
- `.codex-plugin/plugin.json`;
- `.claude-plugin/plugin.json`;
- `shared/gurumaker/runtime/version`;
- `CHANGELOG.md`.

Push an exact `v<version>` tag only after validation. The tag-driven release
workflow builds raw binaries and user archives, verifies native targets,
generates `SHA256SUMS`, and creates the GitHub Release. See
[Runtime distribution and updates](runtime-distribution.md) before changing
targets, wrapper behavior, or release assets.
