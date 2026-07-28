---
name: guru-memory
description: Initialize, recall, or maintain project-owned Wiki, Lens, Evidence, and Method elements, or inspect and explicitly change Guru Mode. Use when no external refresh, analysis of supplied point-in-time external evidence or data, or point-in-time judgment is requested; an explicit request to retain a supplied source is still memory work. Do not use for research, Decisions or outcomes, deliberate lesson extraction, or Guru Pack lifecycle work.
---

# Guru Memory

Maintain project-owned investment memory as ordinary Markdown under
`guru-maker/`. Do not create a workspace for a read-only request; initialize
it only when the user asks to retain memory or install a Guru Pack.

Resolve every linked reference path relative to this loaded `SKILL.md`: a
`../../shared/...` link reaches the sibling `shared/` bundle, never
`skills/shared/...`. Read
[project retrieval](../../shared/gurumaker/references/retrieval.md), [the
workspace contract](../../shared/gurumaker/references/workspace-contract.md),
[Guru Mode](../../shared/gurumaker/references/guru-mode.md), and [the Runtime
contract](../../shared/gurumaker/references/runtime.md) as the requested
operation needs them. Read [the record
contract](../../shared/gurumaker/references/record-contract.md) before changing
a Wiki, Lens, Evidence, or Method.

Use the Runtime search and exact-read path for project memory. Do not enumerate
the memory tree or open remembered elements directly by filesystem path.

Read the stored mode before the first possible memory write and change it only
after an explicit user request. In `auto`, a completed task may make the minimum
durable improvement allowed by the Guru Mode contract. In `off`, create or
update memory only when the user explicitly requests it.

Before creating or updating an element, run a targeted search for its durable
concept; use the bounded reformulation allowed by the retrieval contract when
an alias, identifier, language, or terminology mismatch is plausible. This
pre-write search does not apply to workspace `init`. Improve an existing
semantic Wiki, Lens, or Method in place and keep it independently usable.
Follow the record contract when correcting source-bound Evidence. After a
project-memory edit, run `knowledge check --workspace <project-root>` once
unless the optional hook already returned the result. Route externally
refreshed research, Decisions and Execution Briefs, deliberate lesson
extraction, and Guru Pack management to their public Skills.
