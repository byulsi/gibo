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

For a material Decision, use validation appropriate to its claim, instrument,
data, and horizon. That may mean alternative explanations and disconfirming
evidence, calculation and convention checks, technical regime sensitivity,
quantitative leakage and robustness tests, or portfolio constraint and
scenario analysis. Evidence is adequate only when its scope, period or data
window, and currentness fit the judgment.

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
record contract.

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
