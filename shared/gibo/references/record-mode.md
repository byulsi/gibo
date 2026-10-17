# Record Mode

Record Mode controls whether a host agent may improve local project memory during
an ordinary user-invoked investment task matched by a public Gibo Skill,
including a natural-language task that never names Gibo. It does not
change research quality, read-only retrieval, native host permissions, or the
explicit boundaries for Gibo Packs, Git, external execution, and project
initialization.
It is a behavioral policy for the host agent, not a filesystem permission or
security boundary.

Separate write authorization from memory eligibility. Record Mode determines
whether an otherwise eligible change may be written; the fitting public Skill
and record contract determine what qualifies as investment memory. Neither
Auto nor an explicit request turns unrelated personal preferences,
presentation defaults, instructions, or conversation content into eligible
memory. A convention qualifies only when reviewed as a repeatable investment
Method with an analytical rationale, applicability, and limits.

An initialized project stores the mode in `.gibo/mode`. Read it before the
first possible memory write:

```text
gibo mode --workspace <project-root>
```

Set it only after an explicit user request:

```text
gibo mode auto --workspace <project-root>
gibo mode off --workspace <project-root>
```

`auto` is the default for new and legacy projects. A task-specific user
instruction overrides the stored mode for that task without changing the
project setting.

## Auto

In `auto`, an agent working inside an initialized project may retain a small,
reviewable improvement when completed work produces knowledge likely to matter
again. Eligible changes include:

- source-bound observations or reproducible derived results;
- stable descriptive context, an interpretation, or a repeatable method whose
  scope and limits are understood;
- a material point-in-time judgment or outcome review; and
- a stable correction independently established by the completed work and
  eligible under the normal Wiki, Lens, or Method gates.

The user does not need to mention Gibo, invoke a Skill, name a memory
kind, or ask to save. A matching public Skill should be selected from the
natural request, use relevant project memory when it can materially improve
the task, and apply the durability and kind-selection rules in
[the in-task curation contract](curation.md). This implicit behavior applies
only inside an already initialized project and only during a user-invoked
task.

For material research, a new provenance-bound observation with a cutoff that
changes or materially qualifies a thesis, risk, assumption, or later review
question is retained as one Evidence record by default. A separate save cue is
not required. Explicit no-write instructions, narrow facts, and equivalent
existing Evidence override that default.

Use the minimum sufficient memory change. Prefer improving the matching stable
Wiki, Lens, or Method over creating a duplicate, while preserving point-in-time
Evidence and Decisions. Do not write narrow factual answers, exploratory notes,
transient calculations, speculative inferences, conversation transcripts, or
inferred preferences. Do not initialize a workspace merely to enable Auto.

Auto is in-task curation, not a background process, automatic turn-end capture,
model fine-tuning, or permission to invoke deliberate training. It never makes
Pack lifecycle actions, Git actions, external execution, credentials, or order
details implicit. Extracting an outcome or postmortem lesson remains deliberate
training and requires explicit Train intent; Auto may not infer such a lesson
merely because a task produced a correction.

For an eligible write, follow [the in-task curation contract](curation.md).
Search compact candidates before writing, prefer the kind-specific update
policy, and limit incidental cleanup to no more than five existing local,
mutable records sharing the concept or entity. Reuse a stable nearby folder;
do not create a one-record taxonomy or let paths define identity. Run
`knowledge health` for each touched kind and one final `knowledge check`.

Auto may make small, unambiguous improvements to summaries, useful aliases or
tags, Wiki `see_also` navigation, and obvious metadata or formatting within
that budget. It must not change IDs, delete or merge records, rewrite
historical Evidence or Decisions, migrate a tag scheme, reorganize a whole
collection, or touch installed Pack sources. Health thresholds are advisory
and never authorize those changes.

## Off

In `off`, use existing memory as context but change project memory only when the
user explicitly asks to initialize, retain, save, correct, update, record, or
run a reviewed training write. A request to research, calculate, analyze, or
make a judgment is not by itself a request to persist the result.

An explicit write authorizes the requested record, not adjacent cleanup.
Broader curation in `off` requires its own explicit request. For work beyond
bounded local cleanup, first show a plan and affected IDs; the five-record cap
applies to incidental Auto curation, not to an explicitly reviewed scope.
After any permitted memory edit, follow the record and curation contracts, run
touched-kind health, and run one `knowledge check` unless the optional hook
already returned the result. Briefly report counts of what was added, updated,
or moved.
