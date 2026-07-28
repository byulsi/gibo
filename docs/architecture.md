# Architecture

## Product shape

```text
user request
  -> Codex, Claude Code, or another host agent
  -> one of five public Agent Skills
  -> project Guru Mode gates optional memory writes
  -> shared wrapper resolves the pinned Rust Runtime
  -> Runtime searches or validates project-owned Markdown
  -> optional local memory or Decision edit
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

Wiki, Lens, and Method elements are reusable and independently understandable
and usable. Evidence and Decisions are project-local point-in-time memory and
own the minimal authored relationships.

The tracked `.guru-maker/mode` setting is `auto` or `off`. `auto` is the
default. It permits minimum, reviewable in-task memory improvements inside an
already initialized project. `off` requires an explicit user request for every
memory change. The setting does not authorize initialization, Pack lifecycle,
Git, or external execution.

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
to lexical relevance. One targeted search followed by an exact-ID read is the
ordinary recall path and the Runtime is the only project-memory read interface,
even when a record path or ID is visible. Memory-using work starts with that
query rather than a recursive workspace inventory; inspection of user-supplied
evidence remains scoped to its source path. Every `guru-maker/` record is
project memory regardless of whether it is described as supplied, current, or
Evidence; ordinary external source files live outside that hierarchy. After a
no-match, one bounded normalization is allowed when an alias, identifier,
language, or terminology mismatch is plausible.

An explicit adjacent comparison queries a reusable concept rather than an
entity and may also reformulate it once. There is no catalog fallback, index,
vector store, RAG service, or MCP retrieval surface.

## Relationships and graph

Reusable Wiki, Lens, and Method elements have no structured memory edges.
Evidence may revise earlier Evidence. Decisions may use reusable memory,
support their judgment with Evidence, and revise earlier Decisions.

Markdown is canonical. `decision-graph.html` is a disposable projection of
valid local edges and Evidence source URLs. Pack source nodes and inferred
relationships do not exist.

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
host agent determines research strategy and applies retrieved local elements
as optional context rather than authority. When useful, Guru Research combines
the smallest suitable set of host-available research skills, tools, and data
connectors, then evaluates and normalizes their outputs before synthesis.
Current documents or datasets supplied in the task remain external research
evidence rather than project memory. Skill routing follows task intent, not
asset class or analytical method. Guru Mode separately determines whether the
completed task may improve local memory.
Write authorization and memory eligibility remain separate: neither Auto nor
an explicit request turns unrelated personal or presentation preferences into
investment memory.
Concrete examples default to familiar U.S. instruments, but no routing,
retrieval, record, or Runtime behavior is market-specific.
