# Workspace and Guru Pack contract

Initialize retained memory only after an explicit request:

```bash
gurumaker init .
```

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
  mode
  packs/
    wiki/
    lens/
    method/
```

The five singular directories are the only active memory collections. Each may
organize Markdown elements into nested subject folders, but must not add
another collection wrapper. Search, read, validation, and graph rendering scan
the hierarchy under `guru-maker/`.

`.guru-maker/mode` stores `auto` or `off`. `auto` is the default, including for
legacy initialized projects without the file. Inspect or explicitly change it
through `gurumaker mode [auto|off] --workspace <project-root>`. Guru Mode
controls local memory writes by the host agent; it never makes initialization,
Pack lifecycle, Git, or external execution implicit. Read the [Guru Mode
contract](guru-mode.md) before applying it.

## Kind-specific Pack slots

A Pack contains exactly one of `wiki`, `lens`, or `method`. Its minimal
`guru-pack.json` declares the kind and repository-relative Markdown elements:

```json
{
  "schema_version": 1,
  "kind": "method",
  "elements": ["quality/quality-review.md"]
}
```

Install or update a Pack only after an explicit user request. One hidden source
slot exists per kind. Installation is allowed only when both that slot and the
matching local collection are empty. It stores the exact Git source under
`.guru-maker/packs/<kind>/` and copies the declared elements once into
`guru-maker/<kind>/`.

The hidden checkout is read-only provenance, not memory context. Never search,
read, or apply its files directly. The project-owned local copy may evolve
freely. `pack update` changes only the source checkout, and `pack remove`
preserves local memory.

Pack creation selects local elements of one kind and preserves their paths
within that collection. Packs never contain
Evidence, Decisions, static files, executable code, hooks, scripts, MCP
servers, or order code. Publishing through Git and GitHub is a separate
explicit request.
