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
`tags` only when they materially improve retrieval. Do not add a central tag
registry or a separate `keywords` field.

Every element must be reusable for investment research, analysis, risk, or
decision-making. A personal or presentation preference does not qualify by
itself. When a convention materially affects interpretation, retain it only as
a reviewed Method with its analytical rationale, repeatable use, applicability,
and limits.

| Directory | Required content beyond common frontmatter |
| --- | --- |
| `wiki/` | A connected explanatory concept that improves in place and may use authored `see_also` navigation |
| `lens/` | A small, independent interpretation or hypothesis with assumptions, applicability, limits, and failure indicators |
| `method/` | An independently usable repeatable procedure with inputs, parameters, validation, and limits |
| `evidence/` | An append-oriented, provenance-bound observation or reproducible derived result; requires `source`, with optional `published_at` and `period` |
| `decision/` | A point-in-time judgment preserved through later authored revisions rather than overwritten |

## Wiki authoring

A Wiki may retain reusable explanatory knowledge from science, technology,
industry, value chains, standards, regulation, accounting, or market
conventions even when it does not name a security, provided it is reasonably
likely to improve later investment research, interpretation, or risk analysis.
Do not capture material merely because it is interesting or build a general
encyclopedia dump.

Keep one coherent concept or question, not one isolated fact or an entire
field, in each Wiki element under a stable semantic ID. Start flat when no
durable grouping exists. Add shallow subject folders only after stable groups
emerge; a path organizes files but does not define identity, relevance, or a
relationship. Put searchable meaning in the title, summary, and useful aliases
or tags.

A Wiki may author a small set of exact local Wiki IDs for deliberate
navigation:

```yaml
see_also:
  - wiki:logical-qubits
  - wiki:surface-code
```

`see_also` is optional and Wiki-only. Targets must exist, have kind `wiki`, be
unique, and differ from the containing ID. It is one-way authored navigation:
the Runtime derives incoming backlinks when requested, so do not write a
reverse link merely for symmetry. It does not create a Decision Graph edge,
affect search relevance, recursively traverse, or automatically load target
content. A Wiki remains independently understandable without following it.
Ordinary Markdown links remain untyped source or reading links and are not
parsed as `see_also`. Obsidian-style `[[filename]]` links are not canonical;
use stable IDs in `see_also` rather than filenames or paths for typed Wiki
navigation.

Use descriptive, unique ATX headings whose sections contain enough context to
remain intelligible when retrieved alone. Synthesize the mechanism, scope,
vocabulary, measurements, boundary conditions, constraints, limits, and
investment relevance that are likely to matter again. Cite authoritative
sources in ordinary Markdown when accuracy depends on them.

Keep stable explanatory context in Wiki. Put a current experimental result,
benchmark result, dated vendor roadmap or company claim, or other
time-sensitive state in Evidence with provenance and a cutoff. Put a repeatable
procedure in Method, an interpretation or hypothesis in Lens, and a judgment in
Decision. A Wiki Pack must include every exact `see_also` target declared by
its elements. Pack creation validates this closure but never follows or
automatically adds a target. Source Markdown links remain navigation hints and
do not affect Pack closure.

For derived Evidence, use `source` as the result or primary provenance locator
and record material input sources, data window and frequency, units,
transformations, parameters, benchmark, costs, and artifact, code, or model
version in readable Markdown as applicable. Do not force irrelevant fields onto
simple source observations.

Wiki, Lens, and Method elements improve in place under one stable semantic ID.
Git is their revision history. Evidence and Decisions are point-in-time
elements. Correct a clerical or source-transcription Evidence error in place;
a substantive later correction becomes new Evidence.

## Navigation and relationship rules

Wiki `see_also` references are navigation, not analytical relationships. Lens
and Method have no structured links. Keep every reusable element independently
understandable and usable without hidden context. Ordinary Markdown source
links do not create relationships or automatically load another element.

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
`related` is not part of the contract. Relationship and `see_also` targets
must exist by exact local ID.

Before and after a local edit, follow [the bounded in-task curation
contract](curation.md). Run
`gurumaker knowledge check --workspace <project-root>` once after the complete
edit unless the optional hook already returned the result.
