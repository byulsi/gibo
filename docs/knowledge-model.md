# Knowledge Model

Guru Maker stores investment memory as ordinary project Markdown. A project
owns one collection of each memory kind, and each collection contains any
number of hierarchically organized Markdown elements:

- **Wiki** preserves stable descriptive context, conventions, constraints, and
  explanatory domain knowledge.
- **Lens** states an interpretation, heuristic, or hypothesis with its
  assumptions, applicability, limits, and failure indicators.
- **Method** describes a repeatable research, calculation, model, or review
  procedure with inputs, parameters, validation, and limits.
- **Evidence** preserves a provenance-bound direct observation or reproducible
  derived result at a stated time or data window.
- **Decision** records a material judgment under stated information,
  constraints, and uncertainty.

## Project layout

```text
guru-maker/
  wiki/
  lens/
  method/
  evidence/
  decision/
  decision-graph.html

.guru-maker/
  .gitignore
  mode          auto or off
  packs/
    wiki/       optional installed source checkout
    lens/       optional installed source checkout
    method/     optional installed source checkout
```

Elements are `.md` files anywhere within their single kind collection.
Subject folders may form a hierarchy as the collection grows; a second Wiki,
Lens, Method, Evidence, or Decision collection wrapper is not part of the
contract. The nearest kind directory determines the element kind.

## Element frontmatter

Every element requires:

```yaml
---
id: wiki:semiconductor-advanced-packaging
title: Semiconductor advanced packaging
summary: How chiplets, interposers, and high-bandwidth memory change the value chain.
as_of: 2026-07-27T16:00:00-04:00
---
```

`as_of` contains seconds and either `Z` or an explicit UTC offset. It is the
information horizon, not the file edit time: the reviewed-through time for
reusable memory, the observation or data cutoff for Evidence, and the
information cutoff for a Decision.

Optional fields are `entities`, `aliases`, and `tags`. Include them only when
they materially improve retrieval. Evidence additionally requires `source`
and may include `published_at` and `period`:

```yaml
---
id: evidence:example-foundry-2026q2-capex
title: Example Foundry 2026 Q2 capex guidance
summary: Management retained full-year capex guidance in the Q2 call.
as_of: 2026-07-27T16:00:00-04:00
source: https://example.com/investors/q2-2026-transcript
published_at: 2026-07-25T06:30:00Z
period: 2026-Q2
entities:
  - ticker:EXAMPLE
---
```

Use stable namespaced identifiers such as `ticker:NVDA`,
`isin:US0378331005`, `fx:EURUSD`, `crypto:BTC`, or local `entity:` and `topic:`
IDs. Put quantities and calculations in readable Markdown with their unit,
period or data window, basis, formula, and provenance when those details affect
interpretation. Derived Evidence should also state material input sources,
frequency, transformations, parameters, benchmark, costs, and artifact, code,
or model version as applicable.

Concrete market examples use familiar U.S. listings, but identifiers and
retrieval remain market-neutral.

Wiki, Lens, and Method elements improve in place under one stable semantic ID;
Git preserves their revisions. Evidence and Decisions remain point-in-time
records. Correct only clerical or source-transcription Evidence mistakes in
place. A substantive later correction becomes new Evidence.

## Wiki authoring

Wiki is not limited to financial definitions or facts about named securities.
It may preserve reusable science, technology, industry, value-chain, standards,
regulatory, accounting, or market knowledge when that knowledge is reasonably
likely to improve later investment research, interpretation, or risk analysis.
It is not a general encyclopedia: retain a coherent concept or question that is
likely to matter again, not an isolated fact or an entire field.

For example, a technical Wiki might begin:

```markdown
---
id: wiki:quantum-error-correction
title: Quantum error correction
summary: Why logical qubits require physical redundancy and how that constrains commercialization.
as_of: 2026-07-29T09:00:00+09:00
aliases: [QEC, logical qubit]
tags: [quantum-computing]
---

# Core mechanism

Self-contained explanation of the reusable mechanism...

# Constraints and measurements

Definitions, units, boundary conditions, and common comparison errors...

# Investment relevance

How the mechanism changes questions about scaling, suppliers, and timelines...

# Limits and sources

What this map does not establish, with authoritative source links...
```

Keep one coherent concept or question under one stable semantic ID. Begin with
flat files when no durable grouping exists, then add shallow subject folders
only as stable groups emerge. Paths organize files; they do not define
identity, inherit tags, create relationships, or affect search relevance. Put
searchable meaning in the title, summary, and useful aliases or tags.

The Runtime treats line-start ATX markers as lexical retrieval section
boundaries. Keep intended headings unique and descriptive, avoid line-start
ATX-like comment lines inside fenced examples, and give each section enough
context to remain intelligible when returned alone. Prefer a small number of
substantive sections over tiny fragments, keyword dumps, or link-only pages.
Ordinary Markdown links may cite sources or aid navigation, but they do not
create edges or trigger dependency loading. Pack creation neither follows nor
validates them, so a shareable element may depend only on its own text.

Wiki owns durable explanatory context. A current experimental result, benchmark
result, dated vendor roadmap or company claim, or market state belongs in
Evidence with provenance and a cutoff. A repeatable procedure belongs in
Method, an interpretation or hypothesis in Lens, and a judgment in Decision.
This keeps a stable technical map reusable while requiring later research to
refresh what has changed.

## Minimal relationship model

Wiki, Lens, and Method elements have no structured memory relationships. They
must remain independently understandable, usable, and shareable without hidden
context. Ordinary Markdown source links do not create graph edges or load more
context.

Evidence may only update or contradict earlier Evidence:

```yaml
updates:
  - evidence:earlier-observation
```

A Decision may use Wiki, Lens, and Method elements, cite supporting Evidence,
and update or contradict earlier Decisions:

```yaml
uses:
  - wiki:semiconductor-advanced-packaging
  - lens:quality-growth
  - method:channel-review
supports:
  - evidence:example-foundry-2026q2-capex
updates:
  - decision:example-foundry-prior-review
```

Every edge points from its containing element to the target. `knowledge check`
rejects a missing target, a target of the wrong kind, `related`, or a
relationship on reusable Wiki, Lens, or Method memory.

## Retrieval

```bash
gurumaker knowledge list --kind evidence
gurumaker knowledge search "HBM packaging" --entity ticker:NVDA
gurumaker knowledge read wiki:semiconductor-advanced-packaging --section "Value chain"
gurumaker knowledge check
```

Search scans only local `guru-maker/` elements. Installed source checkouts
under `.guru-maker/packs/` are never searched or placed in context. An
installed Pack seeds local memory once; the local copy then behaves like any
other project-owned element.

Search returns the best matching authored section from each element. When only
element metadata matches, it returns document-level context rather than an
unrelated section. Exact local IDs rank first while authored relationship
targets remain discoverable. Exact titles, aliases, entities, and phrases rank
ahead of complete-token and partial-token matches across tags, summaries,
periods, headings, and content. ASCII words and phrases match at alphanumeric
boundaries, not substrings inside longer words. Unicode lowercase normalizes
case. A space- and punctuation-insensitive comparison is limited to digit-free
Hangul, Han, and Kana queries against concise titles, aliases, tags, and
headings. Common Unicode sign variants are normalized, and signed, decimal,
percentage, and ratio literals must agree exactly. Equal scores prefer the
newer `as_of`.

This remains deterministic lexical retrieval. It does not infer synonyms,
translations, or relationships. The bounded host reformulation below handles a
plausible terminology or language mismatch.

For direct recall, run one targeted query and read a selected exact ID. After a
no-match, one bounded reformulation is allowed when an alias, identifier,
language, abbreviation, or terminology mismatch is plausible. A second
no-match ends recall. An explicit adjacent comparison queries the reusable
concept rather than the named entity. Do not fall back to a catalog, repeated
subqueries, index, RAG service, vector store, or parent-directory scan.

## Guru Mode

`gurumaker mode [auto|off]` reads or explicitly changes `.guru-maker/mode`.
`auto` is the default and permits only minimum, durable, reviewable memory
improvements during an ordinary task in an initialized project. `off` keeps
research and analysis read-only unless the user explicitly asks to retain,
correct, update, record, or run a reviewed training write. Neither mode makes
initialization, Pack lifecycle, Git, or external execution implicit.

When an earlier Decision materially informs a current judgment, search its
exact ID for later Decisions whose authored `updates` or `contradicts` target
that node. Follow only a selected challenger node by node, track visited IDs,
and stop before a repeat or cycle.
