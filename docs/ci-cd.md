# CI/CD

This document is the maintainer map for Guru Maker's GitHub Actions. Runtime
compatibility and installation semantics remain owned by
[Runtime distribution and updates](runtime-distribution.md).

## Continuous integration

`.github/workflows/ci.yml` runs for pull requests, pushes to `main` or
`develop`, and manual dispatch.

The `rust` job:

- checks Rust formatting and Clippy warnings;
- runs all unit, integration, and product-contract tests;
- creates a locked release build;
- parses all plugin and marketplace manifests and checks the five public Skill
  entrypoints plus their shared Runtime bundle;
- validates POSIX wrapper syntax;
- runs the same isolated Runtime fixture as `scripts/test-env.sh`, which
  verifies checksum-gated installation and independent workspaces without
  touching runner-level user data.

The `msrv` job runs the locked test suite with Rust 1.82, the minimum version
declared by the package.

The `host-wrappers` matrix executes the shared wrapper against a native Runtime
on current macOS ARM64 and Windows x86-64 runners. Windows also performs a
complete local fixture install through the PowerShell download path.

Plugin installation and behavioral smokes use isolated temporary homes,
workspaces, and release fixtures, keeping maintainer configuration and
production user memory untouched.

## Release

`.github/workflows/release.yml` is tag-driven. Pushing an exact `v<version>`
tag starts validation and publication.

The workflow:

1. rejects a tag that differs from Cargo, Codex, Claude, or Runtime versions,
   or whose matching `CHANGELOG.md` section is absent, duplicated, or empty;
2. repeats source validation;
3. builds macOS ARM64/x86-64, Linux ARM64/x86-64, and Windows x86-64;
4. executes each natively runnable binary and checks its version;
5. creates raw wrapper-facing binaries and human-facing archives;
6. generates one `SHA256SUMS`;
7. uses the matching versioned changelog section as the GitHub Release body
   and uploads immutable assets.

The publish job alone receives `contents: write`; all other jobs are
read-only. GitHub Releases carries Runtime artifacts, while host marketplaces
carry plugin metadata.

## Preparing a version

Keep these exact values equal during `0.x`:

```text
Cargo.toml
.codex-plugin/plugin.json
.claude-plugin/plugin.json
shared/gurumaker/runtime/version
```

Also move the relevant `CHANGELOG.md` entry out of `Unreleased` when preparing
a public release. That versioned section is the canonical release body, so it
must contain the user-facing changes rather than relying on generated notes.
Validate before tagging:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
sh -n shared/gurumaker/scripts/gurumaker.sh
```

Then commit, push, and push the exact matching tag. After publication, verify
both the rendered Release body and the expected assets. Published tags and
release assets are immutable. If publication partially fails, inspect the
existing release and repair deliberately; a workflow retry deterministically
restores its body from the tagged changelog section, preserves already
published assets, verifies their GitHub SHA-256 digests before uploading only
missing names, and verifies the final name-and-digest set. A same-name digest
mismatch requires deliberate manual repair rather than automatic replacement.

## Failure routing

| Failure | First owner to inspect |
| --- | --- |
| Format, lint, unit, or integration test | `src/` and `tests/` |
| Plugin identity or version mismatch | host manifests and `tests/product_contract.rs` |
| Linux fixture or checksum failure | `shared/gurumaker/scripts/gurumaker.sh` |
| Windows wrapper failure | `shared/gurumaker/scripts/gurumaker.ps1` |
| Target build failure | release matrix target/toolchain setup |
| Missing or renamed release asset | release packaging and `docs/runtime-distribution.md` |
| Missing or incomplete release notes | matching `CHANGELOG.md` section and publish job |
| Host discovers the wrong Skill | marketplace source, plugin cache, and fresh-session smoke |

When adding a host or target, extend only the existing matrix and one
host-native smoke unless a demonstrated compatibility difference requires
more. Host marketplaces remain the update path and GitHub Releases remains the
Runtime registry.
