# Architecture

## Product shape

```text
user request
  -> Codex, Claude Code, or another host agent
  -> one of five public Agent Skills
  -> project Guru Mode gates optional memory writes
  -> shared wrapper resolves the pinned Rust Runtime
  -> Runtime returns compact deterministic candidates
  -> host selects and exact-reads relevant records
  -> optional bounded local memory or Decision edit
  -> Runtime reports health and validates the write
  -> optional offline Decision Graph
```

The repository ships a host-neutral Rust binary, five public Skills, one shared
Runtime bundle, thin host plugin metadata, and optional native-edit hooks. The
Rust core and Markdown contracts own deterministic record, Pack, graph, and
guardrail behavior. Hosts retain reasoning, research tools, delegation, file
editing, and permissions.

## State ownership

Each project owns one hierarchical collection of each kind:

```text
guru-maker/{wiki,lens,method,evidence,decision}/
```

The kinds share a common Markdown envelope but intentionally have different
structures and lifecycles. Wiki is a connected explanatory concept library.
Lens is a small library of bounded interpretations. Method is a catalog of
independently usable procedures. Evidence is append-oriented point-in-time
memory, while Decision preserves the history of point-in-time judgments.

A Wiki may hold reusable scientific, technical, industry, or other domain
knowledge when it can improve future investment work. Its stable semantic ID
owns identity. Shallow subject folders are optional organization, while
descriptive headings delimit independently intelligible retrieval sections;
neither folders nor headings create inherited metadata, graph relationships,
or automatic context expansion. A Wiki may author exact local Wiki IDs in
`see_also`; incoming backlinks are derived on demand. That one-hop navigation
does not affect search relevance, recurse, load context automatically, or
enter the Decision Graph.

The tracked `.guru-maker/mode` setting is `auto` or `off`. `auto` is the
default. It permits minimum, reviewable in-task memory improvements inside an
already initialized project, including when a natural investment request does
not name Guru Maker or a Skill. The host selects the matching public Skill;
there is no fixed router, prompt hook, or background capture. `off` requires an
explicit user request for every memory change. The setting does not authorize
initialization, Pack lifecycle, Git, or external execution.

A project may also have one installed source Pack per reusable kind:

```text
.guru-maker/packs/{wiki,lens,method}/
```

Each source checkout is a clean GitHub repository at an exact commit.
Installation seeds an empty matching local collection once. The source
checkout is thereafter provenance only: it is never searched, loaded into
context, graphed, or edited. Updating it never mutates the project-owned copy.

## Retrieval

Queries recursively scan only local Markdown under `guru-maker/`. Exact IDs,
entities, aliases, titles, summaries, periods, headings, and content contribute
to lexical relevance. Compact candidate search omits bodies and explains its
match; the host selects one to three records and exact-reads only those.
Repeated kind filters support a target lane and, for material named analysis
where it can change the answer, one reusable Wiki/Lens/Method concept lane.
The Runtime is the only project-memory read interface, even when a record path
or ID is visible. Memory-using work starts with that query rather than a
recursive workspace inventory; inspection of user-supplied evidence remains
scoped to its source path. Every `guru-maker/` record is project memory
regardless of whether it is described as supplied, current, or Evidence;
ordinary external source files live outside that hierarchy. After a no-match
or partial-only result, one bounded normalization is allowed when an alias,
identifier, language, or terminology mismatch is plausible.

An explicit adjacent comparison queries a reusable concept rather than an
entity and may also reformulate it once. Explicit related-concept exploration
may request one-hop Wiki neighbors after selecting a Wiki; neighbors are not
ordinary search expansion. Lightweight reranking is an evaluation-gated host
option, not recommended default behavior. Only a host whose representative
ambiguous holdout improves Top-1 selection by at least five percentage points
may delegate at most six compact cards to a read-only subagent; the primary
agent still owns the final selection. There is no fixed router, catalog
fallback, index, vector store, RAG service, or MCP retrieval surface.

## Relationships and graph

Wiki `see_also` and its derived backlinks are navigation, not analytical
relationships. Lens and Method have no structured navigation. Evidence may
revise earlier Evidence. Decisions may use reusable memory, support their
judgment with Evidence, and revise earlier Decisions.

Markdown is canonical. `decision-graph.html` is a disposable projection of
valid local analytical edges and Evidence source URLs. Wiki navigation, Pack
source nodes, and inferred relationships do not exist in that graph.

## Hooks and Runtime

The optional pre-edit hook protects `.guru-maker/packs/`. The post-edit hook
validates only edited local Markdown under `guru-maker/`. Hooks do not search,
inject context, save memory, update Packs, render graphs, or perform Git
actions.

Host plugins share one version-pinned Runtime bundle. Updating the product
selects a new Runtime without changing project memory or Pack sources. Pack
updates remain separate explicit `gurumaker pack update` operations.

## Agent behavior

Memory handles local recall and curation; Research refreshes external
evidence; Decision owns point-in-time judgments and the graph; Train extracts
explicit reviewed lessons; and Packs handles only Pack lifecycle work. The
user does not need to select one of these Skills or name a record kind after
initialization: their natural-language task is matched by the host against the
public Skill descriptions. Material initialized research and judgments begin
with a compact target-memory query; narrow facts and unrelated tasks do not.
The
host agent determines research strategy and applies retrieved local elements
as optional context rather than authority. When useful, Guru Research combines
the smallest suitable set of host-available research skills, tools, and data
connectors, then evaluates and normalizes their outputs before synthesis.
Current documents or datasets supplied in the task remain external research
evidence rather than project memory. Skill routing follows task intent, not
asset class or analytical method. Guru Mode separately determines whether the
completed task may improve local memory.

Authorized writes use bounded in-task curation rather than a user-operated
maintenance workflow. The host classifies the durable payload as stable Wiki,
bounded Lens, repeatable Method, point-in-time Evidence, or material Decision,
then searches before creating and follows that kind's lifecycle. In Auto it
normally makes one primary memory improvement and may tidy no more than five
existing local records in the selected concept's directly relevant
neighborhood. It then reads advisory health for
touched kinds and runs one final validation. Folder thresholds and health bands
guide host judgment but never trigger bulk mutation, ID changes, deletion,
historical rewrites, or background work. Broader work requires a shown plan,
affected IDs, and an explicit request.
Write authorization and memory eligibility remain separate: neither Auto nor
an explicit request turns unrelated personal or presentation preferences into
investment memory.
Concrete examples default to familiar U.S. instruments, but no routing,
retrieval, record, or Runtime behavior is market-specific.
