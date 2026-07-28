# Guru Mode

Guru Mode controls whether a host agent may improve local project memory during
an ordinary user-invoked Guru Maker task. It does not change research quality,
read-only retrieval, native host permissions, or the explicit boundaries for
Guru Packs, Git, external execution, and project initialization.
It is a behavioral policy for the host agent, not a filesystem permission or
security boundary.

Separate write authorization from memory eligibility. Guru Mode determines
whether an otherwise eligible change may be written; the fitting public Skill
and record contract determine what qualifies as investment memory. Neither
Auto nor an explicit request turns unrelated personal preferences,
presentation defaults, instructions, or conversation content into eligible
memory. A convention qualifies only when reviewed as a repeatable investment
Method with an analytical rationale, applicability, and limits.

An initialized project stores the mode in `.guru-maker/mode`. Read it before the
first possible memory write:

```text
gurumaker mode --workspace <project-root>
```

Set it only after an explicit user request:

```text
gurumaker mode auto --workspace <project-root>
gurumaker mode off --workspace <project-root>
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
- a correction or lesson supported by the reviewed interaction.

Use the minimum sufficient memory change. Prefer improving the matching stable
Wiki, Lens, or Method over creating a duplicate, while preserving point-in-time
Evidence and Decisions. Do not write narrow factual answers, exploratory notes,
transient calculations, speculative inferences, conversation transcripts, or
inferred preferences. Do not initialize a workspace merely to enable Auto.

Auto is in-task curation, not a background process, automatic turn-end capture,
model fine-tuning, or permission to invoke deliberate training. It never makes
Pack lifecycle actions, Git actions, external execution, credentials, or order
details implicit.

## Off

In `off`, use existing memory as context but change project memory only when the
user explicitly asks to initialize, retain, save, correct, update, record, or
run a reviewed training write. A request to research, calculate, analyze, or
make a judgment is not by itself a request to persist the result.

After any permitted memory edit, follow the record contract and run one
`knowledge check` unless the optional hook already returned the result. Briefly
report what was retained or changed.
