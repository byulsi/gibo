---
name: train
description: Extract an explicit, user-requested investment lesson from a reviewed scenario, correction, or postmortem, then retain a reusable Lens, Wiki, or Method improvement. Require deliberate training or lesson-extraction intent; Guru Mode Auto and an ordinary natural-language research or review request never invoke this Skill implicitly. Do not use for ordinary recall or research, point-in-time Decisions or outcomes, Guru Pack lifecycle work, scoring, model fine-tuning, or background capture.
---

# Gibo Train

Run only a deliberate session the user requested. Work through the bounded
scenario with the user, identify a reviewed lesson, and make only a
project-owned, reusable memory improvement.

Resolve every linked reference path relative to this loaded `SKILL.md`. The
loaded Skill directory is authoritative: when it is under
`<project>/.agents/skills/`, `../../shared/...` reaches
`<project>/.agents/shared/...`. Never substitute a user-global `.codex` path,
another plugin-cache path, or `skills/shared/...`. Read
[the training session
guide](../../shared/gurumaker/references/training.md) before starting. Read
[project retrieval](../../shared/gurumaker/references/retrieval.md) before
every memory write, [Guru
Mode](../../shared/gurumaker/references/guru-mode.md) to preserve the deliberate
training boundary, [the record
contract](../../shared/gurumaker/references/record-contract.md) before an edit,
[the in-task curation
contract](../../shared/gurumaker/references/curation.md) before retaining the
lesson,
and [the Runtime contract](../../shared/gurumaker/references/runtime.md) before
any `gurumaker` command.

Auto never invokes deliberate training by itself. Once the user has requested
the reviewed training write, both Guru Modes permit that scoped memory change.
That request authorizes only an eligible investment lesson; it does not turn a
personal, aesthetic, or presentation preference into investment memory. A
convention is eligible only when its analytical rationale, repeatable
investment use, applicability, and limits have been reviewed; retain that as a
Method rather than as a preference. Reject ineligible content without
searching or writing memory. In particular, one anecdote cannot support a
universal causal rule. Reject that requested write completely: do not
substitute a safer counter-rule, validation Method, or different memory
record in the same task. Offer the review or counterexample test needed before
any later write.

When the request identifies an ordinary supplied review, correction, or
postmortem by path, read that smallest supplied artifact before accepting its
lesson. Scope source inspection to the identified path or source directory and
never enumerate project memory to find it. If neither the current conversation
nor an explicitly supplied artifact contains enough reviewed facts, ask for
the missing review instead of constructing a lesson.

Before writing, run a targeted search for the reusable concept and read a
compact candidate set, then read a selected match by exact ID; use at most one
bounded reformulation when needed.
Separate reasoning-process quality from outcome luck, test the lesson with an
adjacent case, counterexample, robustness check, or other fit-for-purpose limit
test, and record applicability and limits. Apply the shared kind-specific
curation budget, health, and one-check sequence. Keep the result independently
usable; a Wiki may add exact `see_also` navigation, while Lens and Method do
not gain structured relationships. Route point-in-time Decisions and Pack
lifecycle tasks to their public Skills.
