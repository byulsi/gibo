# In-task curation

Guru Maker keeps project memory useful while an authorized task is already
reading or writing it. Curation is host-owned semantic work around
deterministic Runtime search, health, and validation. It is not a background
process, a fixed agent workflow, or a reason to retain otherwise ineligible
content.

Read the project Guru Mode before the first possible write. In `auto`, curate
only when completed work produces an eligible durable improvement. In `off`,
change only the record explicitly requested by the user unless the user also
requests broader curation. Neither mode authorizes initialization, Pack or Git
operations, or external execution.

In Auto, eligibility does not depend on the user saying Guru Maker, naming a
Skill or memory kind, or asking to save. The host decides from the completed
task. Retain a result only when it is independently understandable, is
source-bound or sufficiently stable, is likely to change a later investment
research or judgment task, and is not already covered by a selected record.
Do not retain a narrow answer, raw exploratory work, unsupported inference,
transient market color, or conversational residue. The user should not have to
operate a maintenance command or choose the record kind.
Rejecting an ineligible requested memory does not authorize a related
corrective Lens, Method, Wiki, Evidence, or Decision write. Suggest the review
needed, but leave memory unchanged unless that review actually occurs.

For material research, a new observation is an eligible durable delta by
default when the answer relies on an identifiable source or reproducible
calculation, has a stated period or cutoff, and changes or materially qualifies
a thesis, risk, assumption, or later review question. In Auto, preserve exactly
one primary Evidence record for that delta unless the user requests no write,
an equivalent source-period-entity record already exists, or the result is a
narrow fact. Do not leave this default contingent on a separate “remember” or
“save” cue.

## Choose the memory kind

Classify the durable payload, not the Skill that produced it or the words the
user happened to use:

| Payload | Kind |
| --- | --- |
| Stable explanatory fact, mechanism, convention, constraint, or domain map | Wiki |
| Interpretation, heuristic, or hypothesis with assumptions, applicability, limits, and falsifiers | Lens |
| Repeatable procedure with inputs, steps or parameters, validation, and limits | Method |
| Source- or calculation-bound observation at a stated time or data window | Evidence |
| Material point-in-time judgment under stated evidence, constraints, and uncertainty | Decision |

Prefer one primary memory improvement per ordinary task. Add a second kind
only when, for example, a current Evidence observation and a stable reusable
synthesis are independently material and each clears the durability gate.
Never create every kind as a template exercise. Research may retain Wiki,
Lens, Method, or Evidence under this rule, but a judgment becomes Decision only
when the user requested a conclusion or outcome review. Deliberate training
remains explicit.

A new or expanded Lens must have reviewed assumptions, applicability, limits,
and a falsifier, and must not generalize from one observation alone. Otherwise
retain the observation as Evidence or report the interpretation as a candidate
without writing it. A Decision is material when the user asks for an allocation
or action conclusion, a consequential choice among alternatives, a revision of
an explicitly selected earlier Decision, or an outcome review intended to
guide future action.
Casual explanation or open-ended exploration without such a conclusion is not
enough to create a Decision.
Research that concludes new evidence changes, weakens, or qualifies a thesis
does not by itself create a Decision. It remains Evidence unless the user also
requests an investment choice or explicit Decision revision.

When a short request has no resolvable referent and no supplied artifact, ask
for the missing subject or material immediately. Do not search project memory,
enumerate files, inspect mode or health, list records, render a graph, or infer
a record kind merely to manufacture context. This early guard runs before any
workspace inventory or Guru Maker command.

## Bounded write sequence

For an eligible write:

1. Search the durable concept or entity with compact candidates, normally
   limiting the applicable kind and returning at most six:

   ```text
   gurumaker knowledge search "<query>" --kind <kind> --candidates --limit 6
   ```

   Reuse an applicable target or relationship search already completed in the
   task. A Decision exact-ID challenger pass satisfies this step; never repeat
   the same concept search solely as a pre-write duplicate check.
2. Use at most one bounded reformulation under the retrieval contract. Select
   one to three exact IDs and read them before deciding whether to improve an
   existing element or create a new one. When updating a selected record, use
   the path returned on its candidate card; do not enumerate the memory tree to
   rediscover that file.
3. Apply the kind-specific lifecycle below. In Auto, keep incidental curation
   to the selected concept and its directly relevant local, mutable
   neighborhood; an entity match alone does not authorize unrelated cleanup.
   Change no more than five existing records in the task.
4. For each touched kind, run
   `gurumaker knowledge health --kind <kind> --workspace <project-root>`.
   Health advisories inform host judgment; they do not authorize extra edits.
5. Run `gurumaker knowledge check --workspace <project-root>` once after the
   complete memory edit unless the optional hook already returned that result.
   This is one knowledge check for the whole bounded write, not one per file.
   Run it as a standalone Runtime invocation so its exit status is observed;
   do not append a Git diff, listing, or unrelated diagnostic to that command.
   After selection and editing, do not run another search, list, or exact read
   merely to verify the write; touched-kind health and this final check are the
   validation.
6. Briefly report counts such as records added, updated, and moved. Do not make
   the user operate a maintenance command for ordinary work.

## Kind-specific lifecycle

- **Wiki** is a connected library of stable explanatory concepts. Prefer
  improving the matching concept. Add only well-supported `see_also` IDs,
  never a written backlink. A clearly oversized touched Wiki may be split only
  when its existing ID remains a useful overview and every new element is
  independently usable.
- **Lens** is a small library of interpretations and hypotheses. Improve an
  existing Lens when the assumption, applicability, and failure conditions
  describe the same interpretation; create another only for a materially
  different scope or falsifier.
- **Method** is a catalog of repeatable procedures. Improve the existing
  procedure unless a variant has independently usable inputs, outputs, or
  validation.
- **Evidence** is append-oriented point-in-time memory. Prevent an exact
  source-period-entity duplicate, correct only clerical or transcription
  errors in place, and otherwise add a new observation with provenance. When
  that observation materially revises earlier Evidence, author the exact
  `updates` or `contradicts` relationship instead of obscuring the lineage.
- **Decision** preserves judgment history. Never rewrite an earlier judgment
  for a later view; add a new Decision with exact authored relationships.

## Placement and local cleanup

Stable IDs, not paths, own identity. Reuse the closest stable subject folder
visible on selected matches. Otherwise keep the element at its kind root. Add
a new subject folder only when at least four elements form a durable topic
group, and keep the hierarchy at no more than two subject levels. Moving a
file must not change its ID, relationships, or search meaning.

Within the five-existing-record budget, Auto may improve summaries, useful
aliases or tags, clear Wiki navigation, and obvious metadata or format defects
on the selected concept's touched neighborhood. Aliases and tags remain
record-authored soft
retrieval hints: do not create a central tag registry, a separate keywords
field, or a global taxonomy migration.

Auto must not change IDs, delete or merge records, rewrite historical Evidence
or Decisions, replace a project-wide tag scheme, perform Pack or Git actions,
or reorganize the whole collection. It must not touch installed Pack source
checkouts. For broader or destructive memory work, first show a plan and the
affected IDs, then perform only the explicitly requested scope. Pack and Git
actions still require their own explicit request and fitting workflow.

Runtime health thresholds are review signals, not automatic mutation triggers.
Existing records are reorganized incrementally when relevant work touches
them. Do not add a daemon, scheduled cleanup, turn-end capture, or bulk
maintenance workflow.
