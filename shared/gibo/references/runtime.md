# Runtime contract

The Gibo plugin exposes five public Skills and one canonical shared bundle.
The shared bundle contains the pinned Runtime, wrappers, and references; it is
not a user-facing Skill and does not run code during plugin installation.

From a loaded public `SKILL.md`, resolve its directory and invoke the shared
wrapper in the plugin's `shared/gibo/` bundle:

```text
sh <public-skill-directory>/../../shared/gibo/scripts/gibo.sh <arguments>
```

On Windows, use:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File <public-skill-directory>\..\..\shared\gibo\scripts\gibo.ps1 <arguments>
```

The wrapper accepts `--runtime-status` for a read-only check and
`--runtime-install` for explicit setup. A normal Gibo command also
installs the runtime on first use, subject to the host's native network and
filesystem permissions.

The Runtime exposes `gibo mode [auto|off] --workspace <project-root>` for
the project-local Record Mode setting. Reading the mode is non-mutating; changing
it requires an explicit user request under the Record Mode contract.

Project-memory reads use `knowledge search`, `read`, `neighbors`, and `health`.
`search --candidates` returns compact metadata without record bodies and
accepts repeated `--kind` values as an OR filter. `neighbors` derives one-hop
Wiki `see_also` and backlink cards without recursion. `health` reports
read-only collection advisories; it never changes memory. Schema and reference
failures remain the responsibility of `knowledge check`.

The plain `runtime/version` file pins the exact Rust Runtime expected by every
public Skill. The wrapper first accepts an exact
`GIBO_RUNTIME_BIN` override, then the managed version, then an exact
`gibo` on `PATH`. If none exists, it downloads the matching release
artifact and `SHA256SUMS`, verifies the digest, and atomically installs the
binary under the user's data directory.

An update never modifies project memory or Gibo Pack source slots. A new plugin version
selects a new versioned runtime directory on its next invocation. If setup
fails, report the error and do not claim that Gibo memory was read or
changed.
