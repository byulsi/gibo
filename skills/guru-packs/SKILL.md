---
name: guru-packs
description: Install, list, inspect, update, remove, create, check, or share a GitHub-backed Wiki, Lens, or Method Pack. Use only for an explicit Pack lifecycle or publishing request; do not use for research, local memory curation, Decisions, or lesson extraction.
---

# Guru Packs

Manage only explicitly requested Guru Pack operations. Each Pack is data-only
and contains exactly one reusable kind—Wiki, Lens, or Method—and a project has
at most one installed source slot per kind. Packs never contain Evidence,
Decisions, static files, hooks, scripts, MCP servers, or order code.

Guru Mode never makes a Pack install, update, removal, creation, or publication
implicit. Every Pack lifecycle and publishing action still requires an explicit
user request.

Resolve every linked reference path relative to this loaded `SKILL.md`: a
`../../shared/...` link reaches the sibling `shared/` bundle, never
`skills/shared/...`. Read [the workspace and Guru Pack
contract](../../shared/gurumaker/references/workspace-contract.md) and [the
Runtime contract](../../shared/gurumaker/references/runtime.md) before using
the CLI.

The CLI namespace is `pack`: invoke
`gurumaker pack <install|list|inspect|update|remove|create|check>`.

Install only into an initialized workspace whose matching local collection and
source slot are empty. Installation keeps the exact Git source read-only under
`.guru-maker/packs/<kind>/` and seeds `guru-maker/<kind>/` once. Ordinary
retrieval uses only the local copy. Updating or removing the source slot never
changes local memory.

Create one kind-specific Pack from explicitly selected local Markdown elements
and preserve their hierarchy within that collection. Pack elements must be
independently self-contained and have no structured memory relationships.

Use system Git credentials without reading, storing, or printing them. Do not
initialize Git, commit, push, publish, or alter local memory unless the user
separately requests that action. Product plugin updates do not update Packs.
