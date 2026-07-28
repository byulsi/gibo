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

## Direct recall

For direct recall, start with one targeted query. If it selects a result, read
that exact ID or the relevant section:

```text
gurumaker knowledge search "<query>" [--kind KIND] [--entity ID]
gurumaker knowledge read <exact-id> [--section NAME]
```

After a no-match, make at most one bounded reformulation only when an alias,
instrument identifier, language, abbreviation, or terminology mismatch is
plausible. Normalize the same intent rather than broadening the question. A
second no-match is definitive.

## Explicit adjacent transfer or comparison

Only when the user explicitly asks to transfer or compare prior learning, query
the reusable concept—the mechanism, constraint, process, or principle—rather
than the named entity. After a no-match, make at most one bounded reformulation
of that concept, then stop. Read a selected result by exact ID.

Neither path falls back to `list`, a catalog, an index, repeated subqueries, or
RAG. Use `list` only for separately requested browsing and `check` only after
local memory changes.

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
