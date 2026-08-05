<div align="center">

# Guru Maker

**Turn your ape agent into an investing guru.**

<img src="./docs/images/guru-maker-hero.webp" alt="A tired chimpanzee in a suit reviews an investment notebook beside stacks of cash" width="960" />

[![CI](https://github.com/monarchjuno/guru-maker/actions/workflows/ci.yml/badge.svg)](https://github.com/monarchjuno/guru-maker/actions/workflows/ci.yml)
![Rust 1.82+](https://img.shields.io/badge/Rust-1.82%2B-000000?logo=rust&logoColor=white)
![Version](https://img.shields.io/badge/version-0.0.4-blue)

</div>

Your ape agent is capable but stateless between investment tasks. It can
produce a sharp investment thesis today, then rebuild the same context from
scratch next month—or ask you to paste its old answer back into the prompt.

Guru Maker gives that work a durable, project-owned memory. It becomes a guru
not because it always knows the answer, but because it remembers what it knew,
why it decided, and what happened next.

## Quick start

### 1. Install

For Codex:

```bash
codex plugin marketplace add monarchjuno/guru-maker
codex plugin add guru-maker@monarchjuno
```

For Claude Code:

```text
/plugin marketplace add monarchjuno/guru-maker
/plugin install guru-maker@monarchjuno
/reload-plugins
```

Start a new task or session after installation.

### 2. Initialize

From the project whose investment memory you want to grow:

```text
$guru-memory Initialize Guru Maker in this project.
```

In Claude Code, use
`/guru-maker:guru-memory Initialize Guru Maker in this project.` Guru Mode
starts as `auto` and retains only small, reviewable improvements. Set it to
`off` when every memory update should require an explicit request.

### 3. Ask naturally

```text
What's changed in this investment since the last review, and does it alter the
thesis?
```

After initialization, the host selects the fitting public Skill and uses
relevant project memory even when you do not mention Guru Maker, a Skill, or a
memory kind. In Guru Mode `auto`, an eligible durable result is also filed as
Wiki, Lens, Method, Evidence, or Decision without a separate save request.
Narrow answers and transient exploration remain conversational.

## Use it naturally

### 1. Give it a map

Start by researching a reusable part of the world:

```text
$guru-research Research the semiconductor manufacturing process. Retain the
reviewed, reusable process and value-chain map as a Wiki, including its scope
and limits.
```

The Wiki can preserve how wafer fabrication, HBM, advanced packaging,
substrates, testing, work in process, qualification lead times, shipments, and
revenue timing fit together. It keeps the stable causal map, not a permanent
view on any company or a stale market claim.

Wiki is not limited to finance or named securities. Reusable scientific and
technical knowledge, such as how quantum error correction constrains logical
qubit scaling, belongs there when it can materially improve later company or
industry research. Current experimental results, benchmark results, dated
vendor roadmaps, and current company claims remain provenance-bound Evidence
rather than timeless Wiki facts.

### 2. Make the next research task better

Later, ask a broad question:

```text
$guru-research Research fictional Atlas Silicon as an AI semiconductor
investment using current sources and relevant project memory.
```

| Without the Wiki | With the Wiki |
| --- | --- |
| Reconstructs the industry or gives a generic growth and valuation review. | Starts from the reviewed process map and spends the task on company-specific evidence. |
| Asks broadly about capacity and orders. | Traces bottlenecks across wafers, HBM, packaging, substrates, test, and qualified output. |
| May repeat old process research. | Reuses the stable map while re-checking every current company and market claim. |

The Wiki does not make Atlas Silicon bullish. It gives the agent a better map
of what to investigate and makes the remaining evidence gaps more precise.

### 3. Turn reviewed experience into a Lens

A material judgment can use that Wiki together with current Evidence:

```text
$guru-decision Should we initiate a position in Atlas Silicon? Record the
judgment, assumptions, risks, and invalidation conditions.
```

After the thesis has had time to meet reality:

```text
$guru-decision Review the outcome. Separate what the process got right from
what was luck, and preserve the later judgment without rewriting the first.
```

Suppose the review finds that demand was correctly identified but value accrued
elsewhere in the supply chain. That is a candidate lesson, not yet a rule. Test
it against an adjacent case and a counterexample:

```text
$guru-train Test whether this review supports a reusable constraint-migration
Lens. Record it only if the interpretation survives a counterexample, with
clear applicability, falsifiers, and limits.
```

If the lesson survives, the Lens can guide future research toward the binding
constraint, how quickly it can move, and where economic value accrues. It
guides the next investigation without dictating its answer.

## From ape to guru

The example above compounds through a reviewable loop:

![Five primate stages progress from a stateless ape through Wiki and Evidence, Decision, and Review to a suited Guru Ape with Lens and Method; a feedback line returns reviewed lessons to future research.](./docs/images/guru-maker-evolution.webp)

**Wiki makes it informed. Lens makes it discerning. Decisions make it
accountable. Reviews make it better.**

This is capability growth, not a mandatory workflow. A factual question can
remain conversational. Research may stop with sourced Evidence or a reusable
Wiki. Only a material judgment becomes a Decision, and only a lesson that
survives review deserves to improve a Lens or Method.

## Generalize without overfitting

**Guru Maker escalates abstraction, not confidence.**

```text
One observation       ──X──> Universal Lens
One winning Decision  ──X──> Permanent rule

Repeated cases + counterexample + process-versus-luck review + explicit limits
                              └──> Reusable Lens or Method
```

- Put current observations and material judgments in point-in-time Evidence and
  Decisions.
- Put stable structure in Wiki and repeatable procedures in Method.
- Promote an interpretation to Lens only with counterexamples, applicability,
  falsifiers, and limits.
- Treat memory as context, not authority. Current claims still require current
  evidence.

The same contract applies across equities, rates, FX, crypto, technical, and
quantitative work. The semiconductor walkthrough is an example, not a required
workflow.

## Memory model

The host agent owns research, judgment, tools, and permissions. Guru Maker owns
the project-memory contract and deterministic operations around it.

Each project keeps five kinds of durable investment memory:

| Kind | Directory | Purpose |
| --- | --- | --- |
| **Wiki** | `guru-maker/wiki/` | Stable descriptive context, conventions, constraints, and domain knowledge |
| **Lens** | `guru-maker/lens/` | An interpretation, heuristic, or hypothesis with scope and limits |
| **Evidence** | `guru-maker/evidence/` | A provenance-bound observation or reproducible derived result at a stated time or data window |
| **Method** | `guru-maker/method/` | A repeatable research, calculation, model, or review procedure |
| **Decision** | `guru-maker/decision/` | A material judgment under stated information, constraints, and uncertainty |

Records stay as ordinary project-owned Markdown. See the
[knowledge model](docs/knowledge-model.md) for schemas, relationships,
point-in-time rules, retrieval, and Guru Mode behavior.

The kinds share a small Markdown envelope but not one lifecycle. Wiki is a
connected explanatory library; Lens is a small set of bounded
interpretations; Method is a procedure catalog; Evidence is an append-oriented
observation log; and Decision preserves judgment revisions.

Wiki records may name a few related local Wiki IDs with `see_also`. Guru Maker
validates those exact references and derives backlinks on demand for explicit
one-hop navigation. It does not infer links, recurse through a graph, add Wiki
links to the Decision Graph, rank search by backlinks, or automatically load
neighbor content.

Ordinary retrieval uses small candidate cards, then exact-reads only the
selected records. Exact IDs, entities, and numbers remain deterministic; the
host agent owns semantic selection. Exact IDs are the internal authority and
control lane for reading and authored relationships, not syntax users must
learn. Lightweight subagent reranking remains an evaluation-gated host option,
not recommended default behavior, until it improves Top-1 selection by at
least five percentage points on a representative ambiguous holdout. No fixed
router is required.

For material work, candidate selection is followed by a bounded activation
pass. If a selected Decision informs the answer, the host exact-reads its full
record, each unique Wiki, Lens, or Method in its authored `uses`, and only
supporting Evidence plausibly material enough to evaluate. It carries forward
only observations actually reused. If no directly read reusable record covers
the single most consequential role, one concept-gap query may select one more
reusable record. The pass never recurses or fans out by kind, and missing,
stale, or inapplicable inputs remain visible.

If a material judgment will create a Decision but the initial mixed
candidate set contains no plausibly applicable Decision, the host makes one
Decision-only fallback search before writing. This protects prior-decision
lineage when higher-ranked Evidence fills the compact result set; it is skipped
when a Decision was already returned or Guru Mode does not permit the write.

In Guru Mode `auto`, an eligible memory write also keeps its immediate
neighborhood tidy: it checks for an existing concept, uses the kind-specific
update policy, reuses stable subject folders, and performs bounded local
cleanup before validation. It never runs in the background or silently
performs bulk moves, ID changes, deletion, history rewriting, Pack work, or Git
work.

Auto works from ordinary natural-language investment requests after the
project is initialized. Material research and judgment tasks reuse targeted
project memory first; completed work is stored only when it is independently
useful and likely to change a later task. The host classifies the durable
payload rather than asking the user to choose: stable explanation becomes
Wiki, a reviewed bounded interpretation becomes Lens, a repeatable procedure
becomes Method, a dated sourced observation becomes Evidence, and a material
point-in-time judgment becomes Decision. Ordinary tasks normally make one
primary memory improvement.

## Public Skills

Host agents discover five public Skills:

| Skill | Use it for |
| --- | --- |
| `guru-research` | Research with current evidence and relevant project memory. |
| `guru-memory` | Initialize, find, retain, correct, and curate memory or change Guru Mode. |
| `guru-decision` | Analyze, record, and review point-in-time judgments. |
| `guru-train` | Turn reviewed scenarios and corrections into reusable improvements. |
| `guru-packs` | Explicitly manage and share GitHub-backed data-only memory packs. |

## Documentation

- [Installation](installation.md)
- [Knowledge model and retrieval](docs/knowledge-model.md)
- [Retrieval scaling and memory lifecycle](docs/retrieval-scaling.md)
- [Decision memory and graph](docs/decision-memory.md)
- [Guru Pack contract](docs/community-plugins.md)
- [Architecture and product boundaries](docs/architecture.md)

## Measured evolution

Markdown records and authored relationships remain canonical. These are
evaluation gates, not promised features. The current retrieval remains
project-scoped lexical candidate search followed by exact-ID reads. It streams
each record into at most one best-section result and uses bounded native scans
for larger collections without adding an index, cache, model artifact, daemon,
or service. Any future derived layer must be rebuildable from local Markdown
and ignore Pack source checkouts. The maintained comparison protocol and
lifecycle gates, including the measured v0.0.4 comparison with the v0.0.3
baseline, are in [Retrieval scaling and memory
lifecycle](docs/retrieval-scaling.md).

- **Fielded lexical index.** Evaluate one only after a representative corpus
  demonstrates a stated scan-latency or peak-memory failure. Markdown remains
  canonical, and missing or stale derived state must fail open to direct scan.

- **Hybrid semantic retrieval.** Only if a labeled project corpus shows lexical
  Recall@K missing a stated target across terminology, language, or conceptual
  variation, evaluate a rebuildable semantic index as a supplement to lexical
  search. Exact IDs, numeric literals, and authored metadata remain
  authoritative. Embeddings or a vector store require an explicit revision of
  the current product boundary before implementation.

Any accepted extension must improve a stated retrieval or navigation metric
without unacceptable precision, context-token, latency, or migration cost.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[development guide](docs/development.md).

## Disclaimer

Guru Maker is software for organizing research and decision records. It does
not provide investment, financial, legal, or tax advice, guarantee outcomes,
or execute trades. Its outputs may be incomplete or incorrect; verify material
facts and sources independently and consult qualified professionals when
appropriate. You remain solely responsible for investment decisions,
compliance, and resulting gains or losses.
