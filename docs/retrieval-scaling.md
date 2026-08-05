# Retrieval Scaling and Memory Lifecycle

Guru Maker keeps local Markdown and authored relationships authoritative.
Search performance, retrieval quality, memory activation, and curation are
separate concerns. A measured failure in one does not automatically justify an
index, embeddings, generated summaries, or a RAG service.

## Current retrieval boundary

Natural-language search returns compact candidate cards, and the host
exact-reads selected records. An exact ID is the stable internal authority and
control address for reading, authored relationships, and Decision lineage; it
is not syntax users must know.

The Runtime searches only project-owned Markdown under `guru-maker/`:

- repeated `--kind` values form one OR filter, and duplicate requested
  collections are discovered only once;
- each Markdown path contributes at most its single best authored-section
  result, and each worker retains only its deterministic top `limit` unique IDs
  before the final merge, ranking, deduplication, and limit;
- collections with at least 256 files use available native parallelism capped
  at eight workers, while smaller scans remain sequential; and
- exact read scans the collection named by a known ID prefix first and falls
  back to other local collections when absent; an unknown prefix scans all
  collections with the same bounded worker policy.

The scorer, `MatchTier`, candidate explanations, entity behavior, exact numeric
rules, and result ordering remain deterministic. There is no persistent cache,
index, model artifact, daemon, service, or second stored representation. Broad
search workers discard full parsed Documents after scoring and merge at most
`workers × limit` compact survivors, so intermediate retained results are
bounded even for a broad match. Search and exact read still retain a linear
worst case in the Markdown scanned.

Material host work may add exact reads after selection: a full selected
Decision, its unique authored `uses` inputs, and plausibly material `supports`
Evidence evaluated for possible reuse. Only observations actually reused are
carried forward. This one-hop activation is explicit host behavior, not
automatic Runtime traversal. Measure its fan-out and latency separately from
the initial search rather than reducing declared-input coverage to hide a
retrieval-cost failure.

## Reproducible evaluation

The opt-in benchmark lives at `scripts/retrieval-benchmark.py`. Inspect its
current interface before a comparison:

```bash
python3 scripts/retrieval-benchmark.py --help
```

Compare a v0.0.4 candidate directly with a clean v0.0.3 Runtime on the same
generated Markdown, machine, filesystem state, and run count. Use isolated
temporary workspaces and report at least:

- source revisions and SHA-256 fingerprints for both binaries, the Runtime
  source, and the benchmark script;
- platform, available CPUs, worker policy, operation-specific worker counts,
  corpus document count, and Markdown bytes;
- warm p50 and nearest-rank p95 latency from at least 20 measured runs, with a
  separate unreported warm-up and alternating baseline/candidate order within
  each operation;
- default section search, compact `--candidates` search, repeated-kind union,
  and exact-ID read; and
- result-order parity for narrow, broad common-term, candidate, repeated-kind,
  and entity-filter searches plus literal retrieval invariants before timing.

Do not convert machine-specific timings into portable test assertions. Record
query selectivity, target position, cold-cache limitations, and whether a
memory profile includes the operating system file cache. If fewer than 20 runs
are used for a smoke, do not present the sample maximum as a stable p95.

### v0.0.4 measured comparison

On 2026-08-05 the maintained benchmark compared v0.0.4 with v0.0.3 commit
`06a840d3e68fb6b61c325fc6fb463b2f6afc7bcc` on macOS 14.6.1 arm64 with eight
available logical CPUs. Each operation used one unreported warm-up and 20 warm
measurements in alternating baseline/candidate order. The generated documents
averaged 5.61 KB; timings therefore include the operating system's warm file
cache and are not cold-cache or cross-machine claims. The run was invoked
through Codex CLI with `gpt-5.6-luna` at `xhigh` reasoning effort.

All literal quality gates passed for both versions at 100, 1,000, and 5,000
documents: phrase, alias, Korean alias, exact ID, signed number, wrong-sign
no-match, true no-match, entity filter, repeated-kind Top-1, compact candidate,
and exact-read fidelity. Output order was identical for narrow, broad,
candidate, broad-candidate, entity-filter, both repeated-kind, and exact-read
comparisons.

The table reports `v0.0.3 p50/p95 → v0.0.4 p50/p95` in milliseconds, followed
by the candidate-to-baseline p50 ratio:

| Documents | Operation | Warm p50/p95 (ms) | p50 ratio |
| ---: | --- | ---: | ---: |
| 100 | Narrow search | 28.388/30.548 → 29.285/32.233 | 1.032 |
| 100 | Broad search | 29.146/31.674 → 30.247/31.299 | 1.038 |
| 100 | Candidate search | 29.168/31.559 → 29.619/31.192 | 1.015 |
| 100 | Repeated-kind search | 14.731/15.058 → 13.700/14.024 | 0.930 |
| 100 | Repeated-kind candidate | 14.945/15.511 → 13.895/14.350 | 0.930 |
| 100 | Exact read | 4.042/4.280 → 2.448/2.798 | 0.606 |
| 1,000 | Narrow search | 243.765/262.841 → 69.716/78.109 | 0.286 |
| 1,000 | Broad search | 251.304/272.930 → 68.748/80.630 | 0.274 |
| 1,000 | Candidate search | 243.109/252.722 → 71.190/76.359 | 0.293 |
| 1,000 | Repeated-kind search | 112.788/116.839 → 32.949/42.547 | 0.292 |
| 1,000 | Repeated-kind candidate | 112.335/116.670 → 31.158/35.381 | 0.277 |
| 1,000 | Exact read | 22.655/25.235 → 5.835/6.264 | 0.258 |
| 5,000 | Narrow search | 1,230.241/1,274.970 → 337.792/423.123 | 0.275 |
| 5,000 | Broad search | 1,243.550/1,331.427 → 326.870/343.263 | 0.263 |
| 5,000 | Candidate search | 1,227.473/1,275.512 → 341.781/408.543 | 0.278 |
| 5,000 | Repeated-kind search | 554.631/574.012 → 134.939/140.201 | 0.243 |
| 5,000 | Repeated-kind candidate | 556.044/574.839 → 131.504/141.281 | 0.236 |
| 5,000 | Exact read | 113.261/119.961 → 11.763/12.173 | 0.104 |

At 100 documents, unfiltered search remained effectively flat, with 1.5% to
3.8% p50 overhead, while filtered search and exact read improved. At 1,000 and
5,000 documents, search p50 fell to 24%–29% of v0.0.3; the 5,000-document exact
read fell to 10.4%. Candidate search used one worker below 256 files and eight
workers for the 1,000- and 5,000-document corpora. Known-prefix exact read used
one worker at 1,000 documents because the selected collection remained below
the threshold, and eight at 5,000.

Reproducibility fingerprints: baseline binary
`76079f678fae2e68974f9746155b7a0bef8ca0946ac4d004eee0c9be6b702dc9`,
candidate binary
`31376c025b975c0e5d14641f4e08df9bc81da819acd7ea8140348d988f9ece95`,
candidate Runtime source
`c5d6bb430588230c0293d812181f6cfa6ef29eca5f3928eef883d4eb09868575`,
benchmark script
`95516a5835317b5c4659a77ccb5f1687c35e4b8ddf101433c60a85463b1e40b6`,
and result JSON
`a1d6634fb1b1470464e2785dbb52bf698a5f4729081235f016f8aa0b025fbdcc`.
These measurements guide this implementation but are intentionally not encoded
as portable latency assertions.

## Retrieval and activation quality

Evaluate search independently from the host's final answer. A labeled project
corpus should include:

- exact ID, entity, alias, title, period, and authored relationship targets;
- signed decimals, percentages, ratios, dates, and distractor numerals;
- terminology, abbreviation, Korean/English variation, and conceptual
  transfer;
- current and superseded Evidence and Decision chains;
- repeated-kind unions, broad common terms, distractor growth, and true
  no-match queries; and
- confirmation that Pack source checkouts never enter results.

Treat exact-ID Top-1, exact numeric agreement, correct kind and entity filters,
deterministic tie breaking, and exact-read fidelity as release invariants.
Measure these dimensions separately:

- required-record recall;
- claim recall within the selected records;
- context precision; and
- context utilization and noise sensitivity in the final answer.

Also report MRR, no-answer precision, obsolete-memory use, returned context
tokens, and final-answer faithfulness. An LLM judge may supplement but never
replace literal checks and human-reviewed relevance.

Sweep candidate budgets `k = 1, 3, 6, 10` and activation depths `0, 1, 2` on
the same labeled tasks. Choose the smallest budget and shallowest depth that
remain near the best measured quality once context precision, utilization,
latency, and noise are considered. This is an evaluation rule, not permission
to exceed the shipped compact limit of six or the one-hop activation contract.

A retrieved record is not yet applied memory. Evaluate host activation with
separate slices for:

- activation false negatives and false positives on material and narrow work;
- declared-input coverage for a relied-on Decision's `uses` and reused
  `supports` targets;
- one-hop compliance, unique-read behavior, and exposure of missing, stale, or
  inapplicable inputs;
- one concept-gap query for the single most consequential uncovered reusable
  role, with no per-kind fan-out or no-match reformulation;
- one Decision-only pre-write fallback when an Evidence-dominated mixed Top-6
  contains no plausibly applicable Decision, with no fallback when a Decision
  was already returned or no Decision will be written;
- constraint consistency when a current Lens, Method, or Wiki changes stale
  Decision prose; and
- grounding of consequential thresholds, allocations, weights, sizing rules,
  and calculations.

Also report useful-read ratio, activation fan-out, retrieval-call count,
context tokens, p50/p95 activation latency, and obsolete-memory use. Include
natural-language requests without Guru Maker or record-kind cues, terse
follow-ups, challenger lineages, true no-match cases, unrelated work, and
fresh-eyes requests.

## Storage and maintenance lifecycle

Corpus growth never authorizes lossy compression or rewriting of canonical
Evidence and Decisions. Their provenance, information horizon, uncertainty,
and challenger history are part of the investment record.

Let Git's pack and delta compression handle canonical Markdown storage. Do not
gzip individual records or replace them with summaries that break direct
search. Any future summary or index remains a regenerable derived aid, never a
substitute for the authored record.

Use this lifecycle:

1. Search compact candidates before writing.
2. Improve an existing Wiki, Lens, or Method when the new material has the
   same stable semantic identity; make an exact duplicate a no-op.
3. Add a narrowly proven alias only after a bounded same-intent reformulation
   recovers a recurring terminology, abbreviation, or language miss. Do not
   generate bulk synonym lists or mutate point-in-time records to chase recall.
4. Preserve materially different point-in-time Evidence and Decisions as new
   records, using only valid authored `updates` or `contradicts` relationships.
5. Keep a small number of descriptive, independently intelligible authored
   sections. The authored section is the retrieval and context unit; avoid both
   whole-document blobs and proposition-per-file fragmentation.
6. Perform duplicate review, consolidation, or higher-level synthesis only in
   an explicit task with a reviewable Markdown diff. Never merge Evidence
   merely because it looks similar, and never delete by age or access count.
7. Pass through the selected raw authored section by default. Query-time
   extraction or compression remains experimental until a fixed-token-budget
   comparison improves support coverage without reducing faithfulness or
   literal, date, source, and uncertainty preservation. Temporary compression
   never edits canonical memory and always retains a raw-section fallback.

This lifecycle makes growth reflect durable information rather than raw
session volume and preserves details needed by an unknown future query.

## Gates for another retrieval layer

These are evaluation gates, not promised features.

### Latency or peak-memory failure

If direct scan misses a stated latency or peak-memory target on a
representative corpus, evaluate a disposable fielded lexical index or
compressed postings before semantic retrieval. Markdown remains canonical.
Derived state must be atomically rebuildable, ignore `.guru-maker/packs/`,
require no daemon, and fail open to the correct direct scan when missing,
corrupt, stale, or incompatible. Exact IDs, numeric literals, and authored
metadata remain the control lane ahead of probabilistic ranking.

If repeated activation reads rather than initial search miss the target,
evaluate a deterministic multi-ID read that resolves selected IDs in one scan
before adding an index, cache, graph expansion, or semantic lane.

### Semantic recall failure

Evaluate hybrid semantic retrieval only when a labeled corpus misses a stated
Recall@K target specifically across terminology, language, or conceptual
variation. Keep exact IDs, numbers, periods, and authored metadata outside
fusion. Compare at fixed K and a fixed context-token budget against the lexical
baseline, including no-answer precision, provenance, currentness, latency, and
migration cost. Embeddings, learned sparse models, or a vector store require an
explicit product-boundary revision and a complete model and index lifecycle.

### Global synthesis failure

Consider a disposable summary projection only when users need corpus-wide
themes and a labeled global-question set demonstrates failure. Any revision
must define atomic rebuild, stale or corrupt fail-open behavior, no-daemon
operation, and a ban on automatic context loading. Generated clusters and
summaries never become canonical memory without explicit review, and inferred
graph edges never become authored relationships.
