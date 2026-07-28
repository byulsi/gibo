# Contributing to Guru Maker

Guru Maker's public distribution is licensed under Apache-2.0. Contributions
require both:

1. acceptance of the [Contributor License Agreement](CLA.md), which lets the
   Project Steward commercially use, sublicense, and relicense contributions
   while contributors retain ownership; and
2. a Developer Certificate of Origin sign-off on every commit:

```bash
git commit -s
```

Opening a pull request through the repository template records CLA acceptance.
The sign-off certifies that the contributor has the right to submit the work
under the repository license. Contributions made on behalf of an employer may
require a separate corporate agreement.

Do not submit a contribution that cannot be licensed under both Apache-2.0 and
the CLA. Maintainers must not merge a pull request until its CLA confirmation
and commit sign-offs are complete.

Keep the host-neutral Rust core, five public Agent Skills, and shared Runtime
bundle canonical. Codex, Claude Code, and future host adapters provide
discovery and installation only; all hosts share the canonical memory, pack,
graph, and execution behavior.

Keep the product scoped to project memory, deterministic CLI operations,
data-only Guru Packs, and the derived graph. Execution integrations stay
external. Keep credentials, portfolio data, private research, and user memory
out of the repository.

Read [AGENTS.md](AGENTS.md) and [docs/architecture.md](docs/architecture.md)
before changing behavior. Run Rust tests and both host plugin validators before
submitting a change.
