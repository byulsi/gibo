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

Optional common fields are `entities`, `aliases`, and `tags`. Include them only
when they materially improve retrieval; tags are soft record-authored hints,
not entries in a central registry, and there is no separate `keywords` field.
Wiki alone may also author `see_also`. Evidence additionally requires `source`
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

The kinds deliberately have different lifecycles. Wiki is a connected concept
library that improves in place. Lens is a small library of independently
applicable interpretations, and Method is a catalog of independently usable
procedures; both improve under stable semantic IDs. Evidence is append-oriented
point-in-time observation, and Decision preserves judgment history through new
authored revisions. Correct only clerical or source-transcription Evidence
mistakes in place. A substantive later correction becomes new Evidence.

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
see_also:
  - wiki:logical-qubits
  - wiki:surface-code
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
create edges or trigger dependency loading. An authored `see_also` item is
different: it is an exact, local, Wiki-only ID used for optional navigation.
The Runtime validates that every target exists, is a Wiki, is not the current
record, and appears only once. It calculates incoming backlinks when requested;
authors do not add reverse links merely for symmetry.

`see_also` does not affect search score, recursively traverse, automatically
load context, or appear in the Decision Graph. A shareable element remains
independently intelligible. A Wiki Pack must declare every `see_also` target
used by its elements. Pack creation validates that closed set but does not
follow or automatically add targets. Ordinary source links have no such Pack
meaning. Obsidian-style `[[filename]]` links are not canonical because stable
IDs, rather than paths or file names, own identity.

Wiki owns durable explanatory context. A current experimental result, benchmark
result, dated vendor roadmap or company claim, or market state belongs in
Evidence with provenance and a cutoff. A repeatable procedure belongs in
Method, an interpretation or hypothesis in Lens, and a judgment in Decision.
This keeps a stable technical map reusable while requiring later research to
refresh what has changed.

## Navigation and minimal relationship model

Wiki `see_also` and derived backlinks form an optional one-hop navigation
surface, not an analytical relationship graph. Lens and Method have no
structured navigation field. Every reusable element remains independently
understandable, usable, and shareable without hidden context. Ordinary
Markdown source links do not create graph edges or load more context.

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
relationship on reusable Wiki, Lens, or Method memory. Wiki `see_also` is
validated separately and never becomes one of these edges.

## Retrieval

```bash
gurumaker knowledge list --kind evidence
gurumaker knowledge search "HBM packaging" --kind wiki --kind method --candidates --limit 6
gurumaker knowledge read wiki:semiconductor-advanced-packaging --section "Value chain"
gurumaker knowledge neighbors wiki:semiconductor-advanced-packaging
gurumaker knowledge health --kind wiki
gurumaker knowledge check
```

Search scans only local `guru-maker/` elements. Installed source checkouts
under `.guru-maker/packs/` are never searched or placed in context. An
installed Pack seeds local memory once; the local copy then behaves like any
other project-owned element.

Search accepts repeated `--kind` values as an OR filter. Its compatible default
output returns the best matching authored section from each element.
`--candidates` instead returns compact candidate cards without section body
text: ID, kind, title, summary, information horizon, path, selected section and
heading path, entities, aliases, tags, authored analytical relationships,
score, `match_tier`, `matched_fields`, and `matched_terms`. Match tiers are
`exact_id`, `exact_metadata`, `phrase`, `all_terms`, and `partial`.

When only element metadata matches, default search returns document-level
context rather than an unrelated section. Exact local IDs rank first while
authored relationship targets remain discoverable. Exact titles, aliases,
entities, and phrases rank ahead of complete-token and partial-token matches
across tags, summaries, periods, headings, and content. Complete query coverage
across concise metadata fields is recognized without treating words scattered
only through a body as a strong match. Tags remain a soft signal, never a hard
filter. ASCII words and phrases match at alphanumeric boundaries, not
substrings inside longer words. Unicode lowercase normalizes case. A space- and
punctuation-insensitive comparison is limited to digit-free Hangul, Han, and
Kana queries against concise titles, aliases, tags, and headings. Common
Unicode sign variants are normalized, and signed, decimal, percentage, and
ratio literals must agree exactly. Equal scores prefer the newer `as_of`.

This remains deterministic lexical retrieval. It does not infer synonyms,
translations, or relationships. The bounded host reformulation below handles a
plausible terminology or language mismatch.

For direct recall, run one targeted candidate query and read one to three
selected exact IDs. For a material named analysis, add one second query over
Wiki, Lens, and Method only when reusable concepts can materially change the
answer. After a no-match or partial-only result, one bounded reformulation is
allowed when an alias, identifier, language, abbreviation, or terminology
mismatch is plausible. A second no-match ends recall. An explicit adjacent
comparison queries the reusable concept rather than the named entity.

Every material research task or requested judgment in an initialized project
begins with one compact target query so existing Evidence and Decisions can
inform the work. Narrow facts, clearly unrelated tasks, and uninitialized
projects skip it. This natural-language routing does not require the user to
name Guru Maker or a public Skill.

Primary-agent selection is the default. Lightweight reranking is only an
evaluation option when all compact candidates are partial and two or more are
meaningfully ambiguous in a material task. Promote it to recommended host
behavior only after a representative ambiguous holdout improves Top-1
selection by at least five percentage points. The subagent ranks at most six
cards, receives no files, tools, or write permission, and returns only selected
IDs, a short reason, confidence, and at most one proposed reformulation. The
primary agent remains responsible.

`knowledge neighbors <wiki-id>` returns compact outgoing `see_also` and
incoming backlink cards for that one Wiki. Use it only for a request to explore
related concepts, effects, comparisons, or transfer. It never returns bodies,
recurses, or expands ordinary search context. Do not fall back to a catalog,
repeated subqueries, index, RAG service, vector store, or parent-directory
scan.

## Collection health

`knowledge health [--kind KIND]` is a read-only, advisory operation. It returns
success even when review is recommended; schema and relationship failures
remain the responsibility of `knowledge check`. JSON reports each requested
kind's document and folder counts, maximum subject depth, review band, and
advisories with stable codes and affected IDs.

The JSON envelope is `{"kinds": [...]}`. A kind has `kind`, `documents`,
`folders`, `max_depth`, `review_band`, and `advisories`; each advisory has
`code` and `ids`. Review bands are `normal`, `organize`, `curate`, and `scale`.
The advisory codes are `deep_folder`, `large_document`, `duplicate_title`,
`duplicate_alias`, `excessive_see_also`, `duplicate_evidence_candidate`, and
`decision_revision_fork`.

The default `organize`, `curate`, and `scale` review thresholds are Wiki
25/75/200; Lens 15/30/100; Method 15/30/100; Evidence 200/500/1000 overall
with entity-level organization at 50; and Decision 50/150/300 overall with
entity-level organization at 10. Advisories identify subject depth beyond two,
documents over 12,000 characters or with more than eight large sections,
normalized title or alias collisions, excessive Wiki navigation, matching
Evidence source-period-entity candidates, and forked Decision revision chains.
More than five authored `see_also` targets is excessive Wiki navigation.
`scale` requests performance evaluation; it does not authorize an index.

## Guru Mode

`gurumaker mode [auto|off]` reads or explicitly changes `.guru-maker/mode`.
`auto` is the default and permits only minimum, durable, reviewable memory
improvements during an ordinary task in an initialized project. The host may
select the fitting Skill, use relevant memory, and retain an eligible result
without the user naming Guru Maker, choosing a record kind, or separately
asking to save. `off` keeps research and analysis read-only unless the user
explicitly asks to retain, correct, update, record, or run a reviewed training
write. Neither mode makes initialization, Pack lifecycle, Git, or external
execution implicit.

Authorized writes follow bounded in-task curation: search compact candidates,
classify the durable payload and apply the kind-specific lifecycle, run health
for touched kinds, then run one knowledge check. An ordinary task normally
makes one primary improvement: stable explanation is Wiki, bounded
interpretation is Lens, repeatable procedure is Method, dated sourced
observation is Evidence, and material judgment is Decision. Incidental Auto
curation may touch no more than five existing local records in the selected
concept's directly relevant neighborhood; an entity match alone does not
authorize unrelated cleanup. Stable folders may be reused, and
a new subject folder is created only for at least four durably related elements
with no more than two subject levels. Auto never changes IDs, deletes or
merges records, rewrites historical Evidence or Decisions, performs global
taxonomy or folder reorganization, or runs in the background. A broader or
destructive scope requires a shown plan, affected IDs, and an explicit request.

When an earlier Decision materially informs a current judgment, search its
exact ID for later Decisions whose authored `updates` or `contradicts` target
that node. Follow only a selected challenger node by node, track visited IDs,
and stop before a repeat or cycle.
