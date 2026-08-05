---
name: guru-maker-development
description: Maintain the Guru Maker repository safely and efficiently. Use for Rust CLI, public Skill, shared Runtime, hook, documentation, CI, or test-harness changes in this repository; do not use for investment-memory work in a user project.
---

# Guru Maker Development

Read `AGENTS.md`, inspect `git status --short`, and preserve changes unrelated
to the request. Treat `src/` and the Markdown contracts as canonical; keep
host metadata, the five public Skills, and shared Runtime wrappers thin.

Use the ownership table and paired-document matrix in `AGENTS.md` to find the
minimum files and tests that must move together. Keep this repository-local
Skill under `.agents/skills/`; never add it to `skills/`, which is the exact
five-Skill public plugin surface.

For release work, prepare one non-empty `CHANGELOG.md` section for the exact
version before tagging. Treat that section as the public GitHub Release body;
do not rely only on generated notes. After the workflow completes, verify the
published body and expected assets before reporting the release complete.

Run `scripts/verify.sh` before handoff for material changes. Use
`scripts/test-env.sh --keep` only to inspect an isolated failure fixture. Do
not use a maintainer's Guru Maker projects, runtime installation, credentials,
or plugin configuration as test inputs.
