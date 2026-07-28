# Runtime Distribution and Updates

This document owns Guru Maker's installation, product-update, release-asset,
and future-host delivery contract.

## Architecture

Each host installs a thin plugin containing the same five public Skills and a
canonical shared bundle. The public Skills invoke one of two bundled wrappers:

```text
Codex plugin ───┐
Claude plugin ──┼─> five public Skills ─> shared bundle ─> wrapper ─> Rust Runtime
future host ────┘
```

The wrappers deliver the Runtime. The Rust binary owns record, search, Pack,
graph, workspace, Guru Mode, and hook decisions. Optional plugin hooks reuse these
wrappers after their lightweight project marker check.

Plugin installation registers the five public Skills. The first actual Guru
Maker command locates an exact compatible Runtime or installs the version
pinned by their shared bundle. Project initialization begins with an explicit
request. Native host filesystem and network permissions remain authoritative.

## Version contract

During the `0.x` series, these versions must match exactly:

- `Cargo.toml`;
- `.codex-plugin/plugin.json`;
- `.claude-plugin/plugin.json`;
- `shared/gurumaker/runtime/version`.

Exact matching avoids a compatibility matrix. A documentation-only plugin
release still receives a matching Runtime release while the product remains
pre-1.0.

The wrapper checks candidates in this order:

1. `GURUMAKER_RUNTIME_BIN`, when explicitly supplied;
2. the plugin-managed versioned Runtime;
3. an exact `gurumaker` found on `PATH`;
4. the matching GitHub Release artifact.

An explicit override participates when its reported version matches the
installed Skill. Every launch uses the exact pinned Runtime.

## Managed paths

POSIX defaults to:

```text
${XDG_DATA_HOME:-$HOME/.local/share}/guru-maker/runtimes/<version>/<target>/gurumaker
```

Windows defaults to:

```text
%LOCALAPPDATA%\GuruMaker\runtimes\<version>\<target>\gurumaker.exe
```

`GURUMAKER_RUNTIME_HOME` overrides the root for controlled testing. Each
version has its own directory, so updates and downgrades coexist while `PATH`
stays unchanged.

## Release asset contract

Tag `v<version>` must contain:

```text
gurumaker-aarch64-apple-darwin
gurumaker-x86_64-apple-darwin
gurumaker-aarch64-unknown-linux-gnu
gurumaker-x86_64-unknown-linux-gnu
gurumaker-x86_64-pc-windows-msvc.exe
SHA256SUMS
```

Human-oriented `.tar.gz` and `.zip` archives accompany the raw binaries. The
wrapper downloads the raw target binary and `SHA256SUMS` from the exact tag,
verifies SHA-256, marks POSIX binaries executable, and moves the verified file
atomically into its versioned destination.

Installation is atomic: checksum and version verification complete before a
Runtime reaches its versioned destination. Host marketplaces drive product
updates, and GitHub Releases provides the Runtime registry.

## Product update lifecycle

```text
host marketplace updates Guru Maker plugin
  -> user starts a new host session
  -> new Skill reads its pinned Runtime version
  -> next Guru Maker command resolves or installs that version
  -> project memory and .guru-maker/mode remain unchanged
```

Codex and Claude Code own plugin discovery and update UX. The Runtime wrapper
manages Guru Maker's versioned Runtime path.

The marketplace catalog points to the repository-local plugin root. A
marketplace refresh and plugin reinstall/update therefore select one coherent
Git snapshot rather than cloning the same repository again through a second
source.

Project memory follows additive record changes. A future breaking record
change includes an explicit, reviewable migration in its release.

Guru Packs have an independent lifecycle. A kind-specific source slot updates
only through an explicit `pack update <wiki|lens|method>` request on a clean
installed checkout. The project-owned local memory seeded at installation is
never changed by that update.

## Release procedure

1. Update the four version owners listed above and `CHANGELOG.md`.
2. Run the validation commands in `docs/development.md`.
3. Commit the release-ready source.
4. Create and push the exact `v<version>` tag.
5. Let `.github/workflows/release.yml` validate, build, execute native target
   smokes, generate `SHA256SUMS`, and create the GitHub Release.
6. Install the released plugin in fresh Codex and Claude Code sessions and
   invoke one command so the wrapper installs the pinned Runtime.

CI creates the release after validation. Published tags and assets are
immutable, and plugin metadata follows verified Runtime assets.

## Adding another host

A future host adapter adds:

- its plugin discovery metadata for the five public Skills;
- an installation/update route;
- a fresh-session discovery smoke.

It reuses the public `skills/guru-*/` trigger surfaces and the canonical
`shared/gurumaker/` bundle, including the Runtime wrappers, Rust release
assets, memory implementation, update path, and version policy.

## Test controls

The wrappers expose three environment variables for deterministic tests:

- `GURUMAKER_RUNTIME_BIN`: exact local Runtime override;
- `GURUMAKER_RUNTIME_HOME`: isolated managed installation root;
- `GURUMAKER_RELEASE_BASE_URL`: alternate release fixture root.

CI exercises an isolated download and checksum verification on Linux, plus
native wrapper-to-Runtime and hook-launcher execution on macOS and Windows.
Release CI also executes every natively runnable built binary before
publication.
