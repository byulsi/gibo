# Changelog

## 0.2.0 - 2026-10-18

- Bundle three read-only data connections in `.mcp.json`: `openbb` (OpenBB
  Platform: U.S. and global prices, macro data, technical calculations),
  `dart` (korean-dart-mcp 0.10.1: OpenDART disclosures, financials, and
  shareholdings), and `krx` (pykrx-mcp at a pinned commit with pykrx 1.2.9:
  KRX investor flows, short selling, indices, and prices).
- Ask for the OpenDART key and KRX login through plugin `userConfig`, stored in
  the system credential store; fall back to environment variables and the
  macOS Keychain.
- Add `mcp/setup.sh` to install the pinned servers into the plugin data folder,
  including a better-sqlite3 override so korean-dart-mcp builds on Node 26.
- Run OpenBB through a relay that hides the 2026-07-28 discovery probe, which
  FastMCP 4 servers reject when Claude Code 2.1.274 sends it alongside the
  initialize handshake. Pin pykrx-mcp to MCP 1.x for the same reason.

## 0.1.0 - 2026-10-17

First Gibo release, based on Guru Maker 0.0.4 and its unreleased changes.

- Rename the product to Gibo for Claude Code: the `gibo` plugin and Runtime,
  `/gibo:research`, `/gibo:memory`, `/gibo:decision`, `/gibo:train`, and
  `/gibo:packs` skills, `gibo/` memory and `.gibo/` settings folders, and
  `GIBO_*` environment variables.
- Rename Guru Mode to Record Mode and Guru Packs to Gibo Packs. Packs use
  `gibo-pack.json`; Packs published with Guru Maker's `guru-pack.json` can
  still be checked and installed.
- Remove Codex plugin metadata, the Codex agent evaluation harness, and the
  contributor license agreement.

The entries below are Guru Maker's history before Gibo.

## Guru Maker Unreleased

- Publish each GitHub Release body from its matching versioned changelog
  section, validate that content as part of the product contract, and verify
  the rendered notes together with release assets before declaring success;
  retries verify existing asset digests before uploading only missing names.

## 0.0.4 - 2026-08-05

- Activate a selected Decision's authored reusable inputs before material
  synthesis, evaluate only plausibly material supporting Evidence and carry
  forward only reused observations, and allow one bounded concept-gap query
  when the most consequential reusable role is still absent, plus one
  Decision-only lineage fallback when mixed candidates contain no plausibly
  applicable Decision.
- Keep Exact IDs as the deterministic selection, relationship, and lineage
  authority while natural-language compact candidates remain the discovery
  path; defer semantic retrieval until a labeled recall failure justifies it.
- Scale direct Markdown retrieval with bounded native parallel scans,
  kind-first discovery, per-record section reduction, and reproducible quality
  and latency evaluation without adding an index, cache, daemon, or service.
- Extend the Codex CLI behavior matrix with natural-language declared-input and
  concept-gap activation cases while preserving the v0.0.3 candidate, health,
  curation, navigation, and large-corpus contracts.

## 0.0.3 - 2026-07-30

- Add stable-ID Wiki `see_also` navigation, deterministic backlinks and
  one-hop neighbor cards, plus closed-link validation for Wiki Packs.
- Add compact explained search candidates, repeated kind filters, and
  read-only collection health advisories while preserving exact-ID, numeric,
  and project-scoped retrieval authority.
- Teach the five public Skills to route ordinary natural-language work into
  bounded, kind-specific Auto curation without requiring users to name Guru
  Maker, while host agents continue to own research strategy and judgment.
- Add 67 Korean, English, short-form, and large-corpus Codex CLI scenarios
  covering save, reuse, stable updates, append-only Evidence and Decision
  lineage, plus labeled retrieval-quality and performance evaluations.

## 0.0.2 - 2026-07-30

- Improve deterministic knowledge search with stronger exact and metadata
  ranking, bounded phrase matching, numeric-literal precision, and compact
  Hangul, Han, and Kana recall.
- Add focused multilingual and retrieval-quality regression coverage.
- Define an LLM-friendly Wiki authoring contract for reusable scientific,
  technical, industry, and investment knowledge while keeping time-sensitive
  results in provenance-bound Evidence.
- Document measured, non-operative evaluation gates for optional Wiki
  backlinks and hybrid semantic retrieval without adding a graph, embedding
  index, or vector store.

## 0.0.1 - 2026-07-29

- Introduce Guru Maker as a host-neutral investment memory layer for Codex,
  Claude Code, and future agent hosts.
- Add the single-binary Rust CLI for project memory, deterministic Markdown
  retrieval and validation, GitHub-backed Guru Packs, and an offline Decision
  Graph.
- Add five public Agent Skills for research, memory, decisions, deliberate
  training, and Guru Packs, backed by one shared Runtime bundle and thin Codex
  and Claude Code plugin manifests.
- Add version-pinned POSIX and PowerShell Runtime wrappers, checksum-verified
  first-use installation, and cross-platform release automation.
- Add optional cross-host edit hooks that protect installed Guru Packs and
  validate local memory without session-wide context injection.
- Define Wiki, Lens, Evidence, Method, and Decision memory with
  compact point-in-time frontmatter and explicit provenance.
- Add a portable Execution Brief for explicit action requests while leaving
  execution to user-selected external capabilities.
