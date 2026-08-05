# Decision Memory and Graph

Guru Maker records material investment judgments and reusable lessons. Narrow
factual answers, status checks, and exploratory notes remain conversational.

## The Reviser principle

**Your past self is evidence, not authority.** A Decision preserves what was
known, assumed, and concluded at a point in time; it does not prescribe the
next decision.

1. **Remember** the earlier context, assumptions, constraints, invalidation
   conditions, and judgment.
2. **Re-test** them against current evidence, counterevidence, and conditions.
3. **Revise** the judgment while preserving the earlier Decision.

A later Decision may `update` or `contradict` an earlier Decision. Follow those
authored challenger edges node by node only when the current answer relies on
the prior judgment. Track visited IDs and stop before a repeat or cycle. Prose
similarity never creates an edge.

Before relying on a selected Decision or challenger, exact-read its full record
and every unique local Wiki, Lens, and Method in its authored `uses`. Exact-read
only supporting Evidence plausibly material to the current question, using the
Decision reasoning and initial candidates to select it. That read evaluates
scope and currentness; carry forward only observations actually reused. This is
one authored hop, not recursive graph expansion; read each target at most once
and expose missing, stale, or inapplicable inputs. If no directly
read reusable record covers the single most consequential role, make the one
concept-gap query allowed by the retrieval contract and exact-read one selected
Wiki, Lens, or Method. Do not fan out by kind or reformulate a gap no-match.

When Guru Mode permits a Decision write and the initial mixed Top-6 contains no
plausibly applicable Decision, make one same-intent Decision-only fallback
search before writing. This is a lineage and duplicate check for a kind that
may have been displaced by higher-ranked Evidence, not another concept query.
Skip it if a Decision was already returned or no Decision will be written.

For a material Decision, use validation appropriate to its claim, instrument,
data, and horizon. That may mean alternative explanations and disconfirming
evidence, calculation and convention checks, technical regime sensitivity,
quantitative leakage and robustness tests, or portfolio constraint and
scenario analysis. Evidence is adequate only when its scope, period or data
window, and currentness fit the judgment. Trace every consequential threshold,
allocation, weight, sizing rule, or calculation to a user constraint, current
Evidence, an explicit calculation, or a directly read and applied reusable
record. Otherwise label it as illustrative or unresolved.

## Decision element

Use one Markdown element per material judgment anywhere within the single
`guru-maker/decision/` hierarchy:

```markdown
---
id: decision:example-corp-entry-2026-07
title: Add Example Corp only below $120
summary: Wait for the stated valuation and thesis conditions.
as_of: 2026-07-27T16:00:00-04:00
uses:
  - wiki:example-corp-business-model
  - lens:quality-growth
  - method:quality-review
supports:
  - evidence:example-corp:q2-renewal-rate
updates:
  - decision:example-corp-prior-review
---

# Add Example Corp only below $120

## Decision

Wait unless the price is at or below $120 and the thesis remains intact.

## Evidence, assumptions, constraints, risks, and invalidation conditions

...

## Sources and as-of

...
```

`uses` targets only local Wiki, Lens, or Method IDs; `supports` targets local
Evidence; and `updates` or `contradicts` targets an earlier local Decision.
Targets must exist. `related` and Pack source references are not part of the
record contract. Author `uses` and `supports` only for records exact-read and
materially applied. A relationship records provenance and does not by itself
trigger Runtime or background loading or recursive traversal; the explicit
material-work activation pass above follows `uses` for one authored hop.

A Decision's `as_of` is its information cutoff, not when the file happened to
be edited. Stale reusable memory may supply context but cannot establish a
current fact.

## Learning

Guru Mode is `auto` by default. In `auto`, a completed material judgment or
outcome review in an initialized project may produce the minimum sufficient
Decision or reusable improvement. In `off`, the same analysis stays
conversational unless the user explicitly asks to record or retain it. Auto is
in-task curation, not background capture or permission for Git, Packs, or
execution.

Reusable Wiki, Lens, and Method elements improve in place under stable IDs.
Evidence and Decisions remain point-in-time history. A substantive later
Evidence correction becomes a new element that updates or contradicts the
earlier Evidence. Changes remain uncommitted until a separate Git request.

Installed Pack sources remain clean under `.guru-maker/packs/`. Their local
copies are ordinary project memory and may evolve independently.

## Derived graph

```bash
gurumaker graph
```

The command scans only local `guru-maker/` elements and writes
`guru-maker/decision-graph.html`. Evidence URLs become source nodes and valid
authored relationships define analytical edges. Pack source nodes are never
included.

The output is self-contained HTML. Markdown remains authoritative and the
graph is a disposable navigation aid.
