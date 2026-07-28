# Memory element contract

Each project has one `wiki`, `lens`, `method`, `evidence`, and `decision`
collection under `guru-maker/`. Elements may use nested subject directories
inside that one collection, and every `.md` file begins with:

```yaml
---
id: lens:quality-companies
title: Quality companies retain pricing power
summary: Tests whether price increases can persist.
as_of: 2026-07-27T09:30:00-04:00
---
```

`id`, `title`, and `summary` are non-empty. `as_of` is RFC 3339 with seconds
and a timezone and represents the information horizon, not the file edit time:
the last reviewed-through time for reusable memory, the observation or data
cutoff for Evidence, and the decision information cutoff for a Decision. The
directory infers kind; omit `kind` and `type`. Add `aliases`, `entities`, or
`tags` only when they materially improve retrieval.

Every element must be reusable for investment research, analysis, risk, or
decision-making. A personal or presentation preference does not qualify by
itself. When a convention materially affects interpretation, retain it only as
a reviewed Method with its analytical rationale, repeatable use, applicability,
and limits.

| Directory | Required content beyond common frontmatter |
| --- | --- |
| `wiki/` | Stable descriptive context, conventions, constraints, or explanatory domain knowledge |
| `lens/` | An interpretation, heuristic, or hypothesis with assumptions, applicability, limits, and failure indicators as appropriate |
| `method/` | A repeatable research, calculation, model, or review procedure with inputs, parameters, validation, and limits |
| `evidence/` | `source` and a provenance-bound direct observation or reproducible derived result; optional `published_at` and `period` |
| `decision/` | Question, judgment, information cutoff, constraints, reasoning, risks, invalidation conditions, and uncertainty |

For derived Evidence, use `source` as the result or primary provenance locator
and record material input sources, data window and frequency, units,
transformations, parameters, benchmark, costs, and artifact, code, or model
version in readable Markdown as applicable. Do not force irrelevant fields onto
simple source observations.

Wiki, Lens, and Method elements improve in place under one stable semantic ID.
Git is their revision history. Evidence and Decisions are point-in-time
elements. Correct a clerical or source-transcription Evidence error in place;
a substantive later correction becomes new Evidence.

## Relationship rules

Wiki, Lens, and Method have no structured memory relationships. Keep every
shareable element independently understandable and usable without requiring
hidden context. Ordinary Markdown source links are allowed and do not create
relationships or automatically load another element.

Evidence may only target earlier Evidence:

```yaml
updates:
  - evidence:earlier-observation
contradicts:
  - evidence:earlier-guidance
```

Decision may use reusable memory, cite Evidence, and revise earlier Decisions:

```yaml
uses:
  - wiki:advanced-packaging
  - lens:quality-companies
  - method:channel-review
supports:
  - evidence:example-corp:q2-renewal-rate
updates:
  - decision:example-corp:prior-review
```

`uses` targets only Wiki, Lens, or Method; `supports` targets only Evidence;
and `updates` or `contradicts` on a Decision target only another Decision.
`related` is not part of the contract. Targets must exist by exact local ID.

After a local edit, run
`gurumaker knowledge check --workspace <project-root>` unless the optional
hook already returned the result.
