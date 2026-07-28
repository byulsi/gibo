# Guru Packs

A Guru Pack is a GitHub transport and provenance unit for exactly one reusable
memory kind: `wiki`, `lens`, or `method`. A project may install at most one
source Pack for each kind.

## Repository contract

The repository root contains `guru-pack.json` and the declared Markdown
elements. Their hierarchy is preserved without an additional Pack wrapper:

```text
guru-pack.json
industry/
  semiconductor-cycle.md
supply-chain.md
```

```json
{
  "schema_version": 1,
  "kind": "wiki",
  "elements": [
    "industry/semiconductor-cycle.md",
    "supply-chain.md"
  ]
}
```

The manifest has no package name, version, or description. The GitHub
repository and exact Git commit are source identity and provenance. Each
element's frontmatter owns its ID, title, summary, and `as_of`.

`elements` is non-empty and contains unique safe repository-relative `.md`
paths. Nested subject directories are allowed and preserved. Path escapes,
symlinks, executable files, static payloads, undeclared files or directories,
malformed elements, duplicate element IDs, and mixed memory kinds are invalid.
Wiki, Lens, and Method elements may not contain structured memory
relationships.

## Installation

```bash
gurumaker pack install https://github.com/owner/wiki-pack.git
```

Installation clones and validates the repository in a temporary location. It
then stores the clean source checkout in the kind slot and seeds the local
collection once:

```text
.guru-maker/packs/wiki/   exact Git source
guru-maker/wiki/          project-owned working elements
```

Installation fails without partial state when either the kind slot already
exists or the corresponding local memory directory is non-empty. Wiki, Lens,
and Method source slots are independent, so a project may install one of each.

Knowledge search, read, check, and the Decision Graph use only the local
working elements. The hidden source checkout is never another memory layer and
never enters agent context.

## Lifecycle

```bash
gurumaker pack list
gurumaker pack inspect wiki
gurumaker pack update wiki
gurumaker pack remove wiki
```

`update` changes only the clean hidden Git source. It never overwrites,
merges, or synchronizes local memory. Applying an upstream change to local
memory is a separate explicit project-memory edit.

`remove` deletes only the hidden source checkout. Local memory remains. A
different Pack of that kind still cannot be installed until the user
explicitly clears or moves the existing local memory.

## Creation and publication

```bash
gurumaker pack create wiki ./wiki-pack \
  --element ./guru-maker/wiki/semiconductor-cycle.md \
  --element ./guru-maker/wiki/supply-chain.md

gurumaker pack check ./wiki-pack
```

Creation copies only explicitly selected elements from the matching local
collection and preserves each path relative to that collection. It does not
initialize Git, commit, push, or publish.
Ordinary Git and GitHub operations follow only after a separate explicit user
request.
