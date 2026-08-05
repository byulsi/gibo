---
name: guru-research
description: Apply Guru Maker's project-memory retrieval and retention contract to material natural-language investment or financial research in an initialized Guru Maker project, even when the user does not mention Guru Maker or saving and another research capability supplies the domain procedure. Host agents and specialized capabilities own research strategy, tool selection, and synthesis. Use explicitly outside an initialized project only when the user names Guru Research. Do not use for memory-only recall, supplied-source capture without refresh or analysis, a requested point-in-time Decision or outcome, deliberate lesson extraction, or Guru Pack lifecycle work.
---

# Guru Research

Investigate the question with the host's chosen research tools and current
sources. Keep project memory project-owned, inspectable, and subordinate to
current evidence. Treat a current document or dataset supplied by the user as
external research evidence; its presence in the project does not make the task
memory-only.

Treat a matching natural-language request as sufficient invocation. Do not
require the user to say research, Guru Maker, a Skill name, a record kind, or
save. Resolve short or elliptical wording from the immediate task, explicitly
supplied artifacts, and targeted project memory. Ask for clarification only
when the unresolved ambiguity would materially change the research question,
requested output, or authorization.
When no referent and no supplied artifact exists, ask immediately without
scanning project files, searching memory, or reading Guru Mode.

Resolve every linked reference path relative to this loaded `SKILL.md`. The
loaded Skill directory is authoritative: when it is under
`<project>/.agents/skills/`, `../../shared/...` reaches
`<project>/.agents/shared/...`. Never substitute a user-global `.codex` path,
another plugin-cache path, or `skills/shared/...`. Read
[project retrieval](../../shared/gurumaker/references/retrieval.md) before
using memory, [research
framing](../../shared/gurumaker/references/research-framing.md) for material
research, [research capability
composition](../../shared/gurumaker/references/research-capabilities.md) when
specialized research support may fit the request, [Guru
Mode](../../shared/gurumaker/references/guru-mode.md) before a possible memory
write, [the workspace
contract](../../shared/gurumaker/references/workspace-contract.md) when project
layout matters, and [the Runtime
contract](../../shared/gurumaker/references/runtime.md) before any `gurumaker`
command. Read [the in-task curation
contract](../../shared/gurumaker/references/curation.md) before retaining any
result.

The host selects any research Skills, tools, and data connectors and decides
how to combine them. Honor that selection and do not act as an agent or tool
router. The host and specialized capabilities own research strategy, tool
selection, domain procedures, evidence evaluation, and synthesis. This Skill
contributes only relevant project-memory retrieval, stale-memory discipline,
and retention boundaries. Apply its compact target-memory query alongside the
host's chosen research path in an initialized project; Guru Mode `off` or a
no-write instruction does not disable that read-only query.

Match the research and validation method to the question rather than assuming
a company, fundamental, causal, technical, or quantitative workflow. Treat
stale memory as context, not a current fact. Search only the project-owned
hierarchy under `guru-maker/`; installed Pack source slots are never research
context. Never filesystem-read a `guru-maker/` record, even just its metadata
before lookup; use Runtime search and exact read for all project memory.

For every material research task in an initialized project, run one compact
target query before external research. Skip it only for a narrow fact or a
clearly unrelated task. After candidate selection, apply the retrieval
reference's bounded activation pass: exact-read an informing Decision, its
authored reusable inputs, and only plausibly material supporting Evidence for
evaluation; carry forward only observations actually reused. Expose missing,
stale, or inapplicable inputs. If the single most consequential
reusable role remains uncovered, run the one allowed concept-gap query and
exact-read its selected result; do not fan out by kind or reformulate a
no-match. This retrieval is useful context; the pre-write candidate search
remains separately required for deduplication when Auto retains a result.

In an initialized workspace, resolve Guru Mode before a possible write. In
`auto`, apply the shared durability gate after completing the research; do not
wait for a separate save request. A new source- or calculation-bound
observation with a period or cutoff that changes or materially qualifies a
thesis, risk, assumption, or later review question must become exactly one
primary Evidence record unless the user requested no write, the result is a
narrow fact, or equivalent source-period-entity Evidence already exists.
Otherwise retain only the minimum stable Wiki, well-bounded Lens, or repeatable
Method improvement likely to matter again. Do not promote a one-off result or
unreviewed inference into reusable memory. In `off`, write only after an
explicit retention request. Read [the record
contract](../../shared/gurumaker/references/record-contract.md) before writing.
Use compact pre-write candidates and apply the kind-specific curation policy;
new current observations remain append-oriented Evidence, while only stable
reviewed knowledge improves reusable memory. Do not create or broaden a Lens
from one observation; require reviewed assumptions, scope, limits, and a
falsifier under the shared gate. Keep incidental cleanup within
the shared five-existing-record budget, then run touched-kind health and one
final knowledge check as specified there. Leave Decisions, Execution Briefs,
deliberate training, and Pack lifecycle work to their respective public
Skills.
If the request asks how evidence changes or qualifies a thesis but does not ask
for an investment choice or explicit prior-Decision review, keep it Research.
When the user explicitly excludes an allocation or investment judgment, never
create a Decision.
