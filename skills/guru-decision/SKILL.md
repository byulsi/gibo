---
name: guru-decision
description: Synthesize a requested point-in-time investment judgment, record a Decision and explicit relationships, review an outcome, or render the Decision Graph. Use also for an Execution Brief after explicit action intent; do not use for general research without a requested judgment, memory curation, explicit lesson extraction, or Guru Pack lifecycle work.
---

# Guru Decision

Synthesize a material investment judgment and, when persistence is permitted,
retain a point-in-time Decision with its evidence, assumptions, constraints,
risks, invalidation conditions, uncertainty, and authored relationships. The
host owns reasoning, sources, tools, and permissions.

Resolve every linked reference path relative to this loaded `SKILL.md`: a
`../../shared/...` link reaches the sibling `shared/` bundle, never
`skills/shared/...`. Read
[project retrieval](../../shared/gurumaker/references/retrieval.md) before
inspecting any prior project memory. Retrieve that memory only through a
targeted Runtime search followed by exact-ID reads; do not enumerate or open
memory files directly. The Runtime is the only project-memory read interface,
even when a record path or exact ID is already visible; do not use filesystem
listing or reading commands as a shortcut. Start memory-using work with the
Runtime query, not a recursive inventory of the workspace; scope any search
for user-supplied evidence to its source location. Every record under
`guru-maker/` is project memory even if the prompt calls it supplied, current,
or Evidence; only ordinary source files outside that hierarchy use filesystem
reading. Read [research
framing](../../shared/gurumaker/references/research-framing.md) only when
gathering or refreshing evidence, [Guru
Mode](../../shared/gurumaker/references/guru-mode.md) before recording a
Decision, [the record
contract](../../shared/gurumaker/references/record-contract.md) before writing,
and [the Decision Graph
contract](../../shared/gurumaker/references/decision-graph.md) for graph work.
Read [the Runtime contract](../../shared/gurumaker/references/runtime.md)
before any `gurumaker` command.

Preserve earlier Decisions; use later authored `updates` or `contradicts`
relationships rather than rewriting history. Re-test material prior judgment
memory and follow its authored challenger chain before relying on it. Challenge
the judgment with the validation appropriate to its claim, data, horizon, and
instrument.

In an initialized workspace, resolve Guru Mode before recording. In `auto`, a
material judgment or outcome review may become the minimum sufficient Decision
without a separate save request. In `off`, analysis remains conversational
unless the user explicitly asks to record it. If the work independently
establishes a reusable improvement in `auto`, keep it separate from the
Decision and apply the Guru Mode threshold; do not turn the Decision outcome
alone into a general lesson. Use only local Wiki, Lens, and Method IDs; support
the Decision only with local Evidence IDs. Pack source checkouts are not memory
context or graph nodes. Render the graph only when relationships change or the
user asks, and report unresolved or skipped records. Create a non-executing
brief only after explicit action intent; then read [the Execution Brief
contract](../../shared/gurumaker/references/execution-brief.md) and write the
requested brief as an ordinary project Markdown artifact. It is not Guru
memory, so Guru Mode does not gate that artifact. Never place an order or
invent execution details.
