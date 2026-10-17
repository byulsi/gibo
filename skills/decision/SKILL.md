---
name: decision
description: Synthesize a requested point-in-time investment judgment, record a Decision and explicit relationships, review an outcome, or render the Decision Graph. Use automatically for natural-language requests asking what investment alternative to choose, buy, sell, hold, allocate, or size, or explicitly revisiting a prior Decision, even when the user does not mention Gibo or a Skill; in Record Mode Auto, a material judgment may be recorded without a separate save request. Use also for an Execution Brief after explicit action intent. Do not use for a research-only thesis update without an investment choice, general research, memory curation, explicit lesson extraction, or Gibo Pack lifecycle work.
---

# Gibo Decision

Synthesize a material investment judgment and, when persistence is permitted,
retain a point-in-time Decision with its evidence, assumptions, constraints,
risks, invalidation conditions, uncertainty, and authored relationships. The
host owns reasoning, sources, tools, and permissions.

Treat a matching natural-language request as sufficient invocation. Do not
require the user to name Gibo, this Skill, a Decision, or a save command.
Resolve short or elliptical wording from the immediate task, supplied
artifacts, and targeted project memory. Ask for clarification only when the
unresolved ambiguity would materially change the judgment or requested action;
never infer action intent merely from a request for analysis.
When no referent and no supplied artifact exists, ask immediately without
scanning project files, searching memory, or reading Record Mode.

Resolve every linked reference path relative to this loaded `SKILL.md`. The
loaded Skill directory is authoritative: when it is under
`<project>/.agents/skills/`, `../../shared/...` reaches
`<project>/.agents/shared/...`. Never substitute a user-global `.codex` path,
another plugin-cache path, or `skills/shared/...`. Read
[project retrieval](../../shared/gibo/references/retrieval.md) before
inspecting any prior project memory. Retrieve that memory only through a
targeted Runtime search followed by exact-ID reads; do not enumerate or open
memory files directly. The Runtime is the only project-memory read interface,
even when a record path or exact ID is already visible; do not use filesystem
listing or reading commands as a shortcut. Start memory-using work with the
Runtime query, not a recursive inventory of the workspace; scope any search
for user-supplied evidence to its source location. Every record under
`gibo/` is project memory even if the prompt calls it supplied, current,
or Evidence; only ordinary source files outside that hierarchy use filesystem
reading. Read [research
framing](../../shared/gibo/references/research-framing.md) only when
gathering or refreshing evidence, [Record
Mode](../../shared/gibo/references/record-mode.md) before recording a
Decision, [the record
contract](../../shared/gibo/references/record-contract.md) before writing,
[the in-task curation
contract](../../shared/gibo/references/curation.md) before recording,
and [the Decision Graph
contract](../../shared/gibo/references/decision-graph.md) for graph work.
Read [the Runtime contract](../../shared/gibo/references/runtime.md)
before any `gibo` command.

Preserve earlier Decisions; use later authored `updates` or `contradicts`
relationships rather than rewriting history. Re-test material prior judgment
memory and follow its authored challenger chain before relying on it. Apply the
retrieval reference's bounded activation pass before synthesis: exact-read an
informing Decision in full, each unique reusable input in its authored `uses`,
and only plausibly material `supports` Evidence for evaluation; carry forward
only observations actually reused. Expose missing, stale, or inapplicable
inputs. If the single most consequential reusable role remains
uncovered, run the one allowed concept-gap query and exact-read its selected
result; do not fan out by kind or reformulate a no-match. Ground consequential
thresholds, allocations, and calculations, then challenge the judgment with
validation appropriate to its claim, data, horizon, and instrument.

For every requested material judgment in an initialized project, begin with
one compact target query and read the selected prior Evidence or Decision
records before gathering more evidence. When prior Decisions and current
Evidence could both matter, keep that first query unscoped instead of splitting
it by kind. Treat “should we act?”, a consequential choice, a revision, or an
outcome review as material.
Do not precede the target query with `rg --files`, `find`, `ls`, `tree`, or an
equivalent broad workspace inventory. Pass `--workspace .` to the Runtime for
discovery, and inspect an ordinary non-memory source path only when the prompt
or a specific result identifies it. The initial target query also satisfies
the curation pre-write search only when it returns a plausibly applicable
Decision. If none is returned and Record Mode permits a new Decision write, run
exactly one same-intent `--kind decision --candidates --limit 6` fallback
before writing; do not reformulate it, and skip it when no Decision will be
written. This bounded lineage check is separate from the one concept-gap
query. Treat an exposure, concentration, causal, interpretation, or risk frame
as a clear Lens gap even when the answer also needs arithmetic or sizing; do
not broaden that role by kind.
Do not turn casual explanation or open-ended exploration into a Decision.
A user-named prior Decision is the target query: use its one exact-ID JSON
search instead of first running a separate broad Decision search. From that
result, exact-read the named Decision and its selected current challenger, then
activate the unique one-hop inputs of each Decision actually used. Do not
re-search a challenger already returned by that pass. The exact-ID challenger
pass also satisfies the curation pre-write search; do not run another broad
topic or duplicate-check query. Use the one remaining search, when needed,
only on the selected challenger to detect a later `updates` or `contradicts`
record. A non-forked named lineage stays within two lineage searches;
activation adds exact reads and may add its one separate concept-gap query when
the consequential reusable role remains uncovered. If multiple direct
challengers remain materially plausible, report the fork instead of silently
dropping a branch to meet the lineage budget. After writing, rely on the
standalone final knowledge check rather than exact-reading the new file merely
to verify the write.
A question about whether new evidence changes, weakens, or qualifies a thesis
remains Research when it does not request an investment choice or explicitly
revisit a stored Decision. If the user excludes an allocation or investment
judgment, do not create a Decision.

In an initialized workspace, resolve Record Mode before recording. In `auto`, a
material judgment or outcome review may become the minimum sufficient Decision
without a separate save request. In `off`, analysis remains conversational
unless the user explicitly asks to record it. If the work suggests a reusable
Wiki, Lens, Method, or Evidence improvement, report that candidate separately
and route any write through the fitting Memory, Research, or Train Skill; do
not write it through Decision or turn an outcome alone into a general lesson.
Use only local Wiki, Lens, and Method IDs; support the Decision only with local
Evidence IDs. Pack source checkouts are not memory context or graph nodes. Wiki
`see_also` navigation and derived backlinks are not Decision relationships or
Decision Graph edges. Apply the bounded kind-specific curation contract to the
new Decision, never to rewrite earlier judgments; then run Decision health and
one final knowledge check. Render the graph only when relationships change or
the user asks, and report unresolved or skipped records. Create a non-executing
brief only after explicit action intent; then read [the Execution Brief
contract](../../shared/gibo/references/execution-brief.md) and write the
requested brief as an ordinary project Markdown artifact. It is not Gibo
memory, so Record Mode does not gate that artifact. Never place an order or
invent execution details.
