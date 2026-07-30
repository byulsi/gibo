# Project retrieval

Use the Runtime search and exact-read commands for project memory. Do not
enumerate the memory tree or open remembered elements directly by filesystem
path. This restriction does not apply to user-supplied source documents or
datasets used as external evidence.

The Runtime is the only project-memory read interface, even when a record path
or exact ID is already visible. Do not use filesystem listing or reading
commands as a shortcut to discover or load `guru-maker/` records.

Start memory-using work with the Runtime query, not a recursive inventory of
the workspace. When user-supplied evidence must be located, scope filesystem
inspection to the supplied path or source directory so it cannot enumerate
project memory incidentally.

Every record under `guru-maker/` is project memory even when a prompt calls it
supplied, current, or Evidence. Read it through the Runtime. User-supplied
external evidence uses ordinary project files outside that hierarchy and may
be read with the host's normal file or research tools.

Work only in the current project. Do not scan parent directories or inspect
anything under `.guru-maker/packs/` during ordinary analysis; those source
slots are provenance, not memory context.

## Direct recall with compact candidates

Start recall with one targeted compact-candidate query, returning no more than
six records. Repeated `--kind` values are an OR filter:

```text
gurumaker knowledge search "<query>" [--kind KIND]... [--entity ID] \
  --candidates --limit 6
gurumaker knowledge read <exact-id> [--section NAME]
```

Candidate output omits body text and carries the record ID, kind, title,
summary, information horizon, path, selected section and heading path,
entities, aliases, tags, authored analytical relationships, score,
`match_tier`, `matched_fields`, and `matched_terms`. Select one to three exact
IDs, then use exact read for the context actually needed. Exact IDs, entities,
and numeric literals remain deterministic authority; tags are only soft
signals. `see_also` does not affect ordinary search. For an authorized update,
reuse the selected candidate card's `path`; never list or scan the memory tree
to locate that record again.

For direct recall, use one target query. For a material analysis about a named
case, add one reusable-concept query over Wiki, Lens, and Method only when that
knowledge can materially change the answer. Do not run that second lane for a
narrow fact or merely because reusable memory exists.

In an initialized project, every material research task or requested judgment
starts with one compact target query so prior Evidence and Decisions can inform
the work instead of serving only as a pre-write duplicate check. Skip this
query for a narrow fact, a clearly unrelated task, or an uninitialized project.
The optional reusable-concept lane remains conditional as described above.

After a no-match or partial-only candidate set, make at most one bounded
reformulation only when an alias, instrument identifier, language,
abbreviation, or terminology mismatch is plausible. Normalize the same intent
rather than broadening the question. A second no-match is definitive.
For a language mismatch, translate the same concept once into the likely
record language or canonical domain term. Do not repeat the unchanged query
and count that as the one reformulation.

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
task is material. The host owns this evaluation; Guru Maker does not provide a
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
gurumaker knowledge neighbors <selected-wiki-id> [--json]
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
gurumaker knowledge search "<selected-exact-id>" --json
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
Exact-read the named Decision, the selected current challenger, and at most one
current Evidence record directly supported by that challenger. Do not load all
historical supporting Evidence. Use a second search only when needed to check
whether that selected challenger itself has a later authored challenger. A
non-forked named lineage stays within two searches total and one to three
pre-write exact reads. If multiple direct challengers remain materially
plausible, report the fork instead of silently dropping a branch to meet that
budget. A final knowledge check validates a new Decision; do not exact-read
the just-written record as a verification step.
