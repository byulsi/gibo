# Project retrieval

Use the Runtime search and exact-read commands for project memory. Do not
enumerate the memory tree or open remembered elements directly by filesystem
path. This restriction does not apply to user-supplied source documents or
datasets used as external evidence.

The Runtime is the only project-memory read interface, even when a record path
or exact ID is already visible. Do not use filesystem listing or reading
commands as a shortcut to discover or load `gibo/` records.

Start memory-using work with the Runtime query, not a recursive inventory of
the workspace. When user-supplied evidence must be located, scope filesystem
inspection to the supplied path or source directory so it cannot enumerate
project memory incidentally.

Do not run `rg --files`, `find`, `ls`, `tree`, or an equivalent broad workspace
inventory to discover whether project memory or source files exist. The
Runtime query, with `--workspace .` when appropriate, is the discovery step.
Load only the Skill and contract references required for the task before that
query; inspect a non-memory source path later only when the prompt or a
specific tool result identifies it.

Every record under `gibo/` is project memory even when a prompt calls it
supplied, current, or Evidence. Read it through the Runtime. User-supplied
external evidence uses ordinary project files outside that hierarchy and may
be read with the host's normal file or research tools.

Work only in the current project. Do not scan parent directories or inspect
anything under `.gibo/packs/` during ordinary analysis; those source
slots are provenance, not memory context.

## Direct recall with compact candidates

Start recall with one targeted compact-candidate query, returning no more than
six records. Repeated `--kind` values are an OR filter:

```text
gibo knowledge search "<query>" [--kind KIND]... [--entity ID] \
  --candidates --limit 6
gibo knowledge read <exact-id> [--section NAME]
```

Candidate output omits body text and carries the record ID, kind, title,
summary, information horizon, path, selected section and heading path,
entities, aliases, tags, authored analytical relationships, score,
`match_tier`, `matched_fields`, and `matched_terms`. Select one to three exact
IDs, then use exact read for the context actually needed. An exact ID is the
Runtime's internal authority and control address for selection, relationships,
and lineage; it is not syntax the user must know or type. Natural-language
search discovers the candidate, and the host passes the selected ID to exact
read. Exact IDs, entities, and numeric literals remain deterministic authority;
tags are only soft signals. `see_also` does not affect ordinary search. For an
authorized update, reuse the selected candidate card's `path`; never list or
scan the memory tree to locate that record again.

For direct recall, use one target query. In an initialized project, every
material research task or requested judgment starts with one compact target
query so prior Evidence and Decisions can inform the work instead of serving
only as a pre-write duplicate check. Skip this query for a narrow fact, a
clearly unrelated task, or an uninitialized project. When prior Decisions and
current Evidence could both materially affect the work, keep this first query
unscoped instead of issuing separate Decision and Evidence queries.

For a new Decision produced by that material task, this initial target query
satisfies the curation contract's pre-write search only when its cards include
a plausibly applicable Decision. When no such Decision card is returned and
Record Mode permits a new Decision write, run exactly one fallback with the same
judgment intent before writing:

```text
gibo knowledge search "<judgment intent>" \
  --kind decision --candidates --limit 6
```

This protects lineage when higher-ranked Evidence fills the mixed top six. Do
not run it when a plausible Decision was already returned or no Decision will
be written. Exact-read and activate an applicable fallback result before
proceeding. This bounded duplicate-and-lineage lane is distinct from the one
concept-gap query below; neither resets or substitutes for the other.
Do not reformulate or repeat this Decision-only fallback after a partial or
no-match result. The generic bounded reformulation below applies to the initial
target query, not to this fallback or the concept-gap query.

After a no-match or partial-only candidate set, make at most one bounded
reformulation only when an alias, instrument identifier, language,
abbreviation, or terminology mismatch is plausible. Normalize the same intent
rather than broadening the question. A second no-match is definitive.
For a language mismatch, translate the same concept once into the likely
record language or canonical domain term. Do not repeat the unchanged query
and count that as the one reformulation.

## Activate selected memory for material work

Candidate selection is discovery, not application. After selecting the initial
records, use this bounded activation pass for material research or judgment:

1. When a selected Decision materially informs the work, exact-read its full
   record so its reasoning and relationship metadata are visible. Do not use a
   section-only read as a substitute. Exact-read each unique local Wiki, Lens,
   or Method in its authored `uses`, unless that exact record was already read.
2. From the Decision's reasoning and the initial candidate cards, identify
   `supports` Evidence plausibly material to the current question and exact-read
   only those records to evaluate their observation, scope, period, provenance,
   and currentness. An evaluation read does not imply reuse. Do not load every
   supporting Evidence record; carry forward only an observation the answer
   actually reuses, compares, or treats as current.
3. Apply a declared input only when it fits the current question and conditions.
   Expose a missing, unreadable, stale, or inapplicable input instead of silently
   reconstructing or replacing it.

This is one authored hop from each selected Decision. Track exact IDs, read
each target at most once, and do not recurse from reusable inputs or Evidence,
follow ordinary Markdown links, expand Wiki neighbors, or inspect Pack source
checkouts. Apply the same pass before relying on a selected challenger. This is
explicit host retrieval, not Runtime context injection or graph traversal.

“Bounded” limits graph depth, duplicate reads, and search lanes; it does not
hide an authored `uses` fan-out behind a hard universal count. Keep a new
Decision's `uses` to the smallest set materially applied. If an existing
Decision's direct inputs exceed the host's stated context or latency budget,
do not sample them or partially rely on the Decision: expose it as not fully
activated and as a curation or multi-ID-read candidate.

Before synthesis, identify the single most consequential reusable role the
work depends on: an interpretation or exposure frame, a calculation or review
procedure, or a stable mechanic or constraint. A gap remains when no directly
read Wiki, Lens, or Method covers that role; a plausible rule improvised from
Evidence does not close it.

For that one gap, make exactly one compact concept query and exact-read one
selected result. Use the matching kind when the role is clear—Lens for an
interpretation, causal or exposure frame; Method for a repeatable calculation,
procedure, or review discipline; and Wiki for a stable mechanic, definition,
or constraint:

Treat an exposure, concentration, causal, interpretation, or risk frame as a
clear Lens role even when downstream work includes arithmetic or sizing. Use
Method only when the missing reusable role is the repeatable calculation or
procedure itself. Use the repeated-kind form only when the role—not merely its
downstream application—is genuinely ambiguous.

```text
gibo knowledge search "<gap concept>" \
  --kind lens --candidates --limit 6
```

When the kind is genuinely ambiguous, use repeated kind filters as one OR
query:

```text
gibo knowledge search "<gap concept>" \
  --kind wiki --kind lens --kind method --candidates --limit 6
```

Do not query once per kind, reformulate a no-match, or run this lane for a
narrow fact, generic inspiration, or a role already covered by a directly read
reusable record. Treat the gap pass as the one reusable-concept query allowed
implicitly for material coverage.

Before finalizing material work, trace every consequential threshold,
allocation, weight, sizing rule, or calculation to a user constraint, current
Evidence, an explicit calculation, or a directly read and applied reusable
record. Otherwise label it as an illustrative assumption or unresolved input,
not memory-grounded precision. A new Decision's `uses` and `supports` list only
records that were directly read and materially applied.

## Explicit adjacent transfer or comparison

Only when the user explicitly asks to transfer or compare prior learning,
query the reusable concept—the mechanism, constraint, process, or
principle—rather than the named entity. After a no-match or partial-only
result, make at most one bounded reformulation of that concept, then stop. Read
a selected result by exact ID.

Neither path falls back to `list`, a catalog, an index, repeated subqueries, or
RAG. Use `list` only for separately requested browsing and `check` only after
local memory changes.

## Optional candidate reranking

Primary-agent selection is the default. A host evaluating delegation may ask a
lightweight subagent to rerank at most six candidate cards only when all top
matches are `partial`, two or more cards are meaningfully ambiguous, and the
task is material. The host owns this evaluation; Gibo does not provide a
fixed router or manager.

Give the subagent only the user intent and compact cards, with no record body,
filesystem access, tools, or write permission. Ask for selected exact IDs, one
short reason per selection, confidence, and at most one same-intent
reformulation. The primary agent verifies and reads the selected IDs and
remains responsible for the answer. Hosts without a suitable subagent do this
selection directly. Promote delegation to recommended host behavior only when
a representative ambiguous holdout shows at least a 5 percentage-point Top-1
selection improvement over direct primary-agent selection.

## Wiki one-hop navigation

Use Wiki navigation only when the request calls for related concepts, effects,
comparisons, or transfer after a Wiki has already been selected:

```text
gibo knowledge neighbors <selected-wiki-id> [--json]
```

This returns compact outgoing `see_also` and incoming backlink cards. Select
any useful neighbor by exact ID. Do not recurse, turn a backlink into an
analytical relationship, or load neighbors automatically during ordinary
search.

## Re-test material judgment memory

Your past self is evidence, not authority. When a selected Decision materially
informs a current judgment, remember its reasoning, re-test its assumptions
and invalidation conditions, then revise the conclusion if needed. A Lens is
independent reusable context and does not have a challenger chain.

Track visited IDs, then pass through authored challengers one node at a time.
For the current node, run one exact-ID JSON search:

```text
gibo knowledge search "<selected-exact-id>" --json
```

Read only later records whose authored `updates` or `contradicts` relationship
targets that exact ID; the later record always points to the earlier target.
Ignore prose matches, other edge types, and the current record itself. Read
each selected challenger by exact ID. If the answer relies on that challenger,
make it the current node and repeat the exact-ID pass. Stop before revisiting
an ID, which also stops a malformed cycle. Do not add this pass to a narrow
factual lookup.

When the user already names the starting Decision, this exact-ID pass replaces
the ordinary broad target query and satisfies the curation pre-write search.
Exact-read the named Decision and any selected current challenger, then apply
the one-hop activation pass to each Decision actually used. Evaluate only
plausibly material supporting Evidence, and carry forward only observations
reused in the answer rather than all historical support. Use a second search
only when needed to check whether that selected challenger itself has a later
authored challenger. A non-forked named lineage stays within two lineage
searches. Activation adds unique exact reads and, only when its single most
consequential reusable role remains uncovered, may add the separate one
concept-gap query defined above. That gap query is not a challenger or lineage
search. If multiple direct challengers remain materially plausible, report the
fork instead of silently dropping a branch to meet the lineage budget. A final
knowledge check validates a new Decision; do not exact-read the just-written
record as a verification step.
