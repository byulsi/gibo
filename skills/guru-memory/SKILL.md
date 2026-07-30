---
name: guru-memory
description: Recall or maintain project-owned Wiki, Lens, Evidence, and Method memory, initialize a workspace, or inspect and explicitly change Guru Mode. Use automatically in an initialized Guru Maker project when a natural-language investment request can materially benefit from existing local memory or produces eligible stable knowledge, even if the user does not mention Guru Maker, a Skill, or saving. Use also for a short follow-up that clearly refers to saved investment memory but omits its subject, so the missing referent is requested before searching. Use when no external refresh, analysis of supplied point-in-time external evidence or data, or point-in-time judgment is requested; an explicit request to retain a supplied source is still memory work. Do not use for research, Decisions or outcomes, deliberate lesson extraction, or Guru Pack lifecycle work.
---

# Guru Memory

Maintain project-owned investment memory as ordinary Markdown under
`guru-maker/`. Do not create a workspace for a read-only request; initialize
it only when the user asks to retain memory or install a Guru Pack.

Treat a matching natural-language request as sufficient invocation. Do not
require the user to name Guru Maker, this Skill, a record kind, or a save
command. In an initialized project, use relevant memory when it can materially
improve the answer. A short or elliptical request may be resolved from the
immediate task, explicitly supplied artifacts, and targeted project-memory
search; ask a clarifying question only when the unresolved ambiguity would
materially change the answer or authorized action.
When no referent and no supplied artifact exists, ask immediately without
scanning project files, searching memory, reading Guru Mode, inspecting health,
listing records, or rendering the Decision Graph.

Resolve every linked reference path relative to this loaded `SKILL.md`. The
loaded Skill directory is authoritative: when it is under
`<project>/.agents/skills/`, `../../shared/...` reaches
`<project>/.agents/shared/...`. Never substitute a user-global `.codex` path,
another plugin-cache path, or `skills/shared/...`. Read
[project retrieval](../../shared/gurumaker/references/retrieval.md), [the
workspace contract](../../shared/gurumaker/references/workspace-contract.md),
[Guru Mode](../../shared/gurumaker/references/guru-mode.md), and [the Runtime
contract](../../shared/gurumaker/references/runtime.md) as the requested
operation needs them. Read [the record
contract](../../shared/gurumaker/references/record-contract.md) before changing
a Wiki, Lens, Evidence, or Method, and [the in-task curation
contract](../../shared/gurumaker/references/curation.md) before any such write.

Use the Runtime search and exact-read path for project memory. Do not enumerate
the memory tree or open remembered elements directly by filesystem path.
Never run `rg`, `grep`, `find`, `ls`, `cat`, `sed`, or another filesystem
command against `guru-maker/`, even for metadata or to locate an ID; use
Runtime search and read. Do not run `init` in an already initialized workspace
or after a successful mode or knowledge command; a no-match is not evidence
that initialization is missing.

Read the stored mode before the first possible memory write and change it only
after an explicit user request. In `auto`, a completed task may make the minimum
durable improvement allowed by the Guru Mode contract. In `off`, create or
update memory only when the user explicitly requests it.

Before creating or updating an element, run a targeted search for its durable
concept with compact candidates; use the bounded reformulation allowed by the
retrieval contract when an alias, identifier, language, or terminology
mismatch is plausible. For a language mismatch, translate the same concept
once into the likely record language or canonical domain term; repeating the
unchanged query is not a reformulation. This pre-write search does not apply to workspace
`init`. Own the kind-specific, bounded local curation sequence: improve a
matching Wiki, Lens, or Method when appropriate, preserve point-in-time
Evidence, choose the record kind from the semantic payload rather than the
user's wording, and keep every reusable record independently usable. Wiki
`see_also` is optional one-hop navigation, never a reason to load or edit all
neighbors. Follow the curation contract for folder placement, the Auto
five-existing-record budget, explicitly reviewed broader scopes, health, final
validation, and reporting. Route externally refreshed research, Decisions and
Execution Briefs, deliberate lesson extraction, and Guru Pack management to
their public Skills.
