# Runtime contract

Guru Maker plugins expose five public Skills and one canonical shared bundle.
The shared bundle contains the pinned Runtime, wrappers, and references; it is
not a user-facing Skill and does not run code during plugin installation.

From a loaded public `SKILL.md`, resolve its directory and invoke the shared
wrapper in the plugin's `shared/gurumaker/` bundle:

```text
sh <public-skill-directory>/../../shared/gurumaker/scripts/gurumaker.sh <arguments>
```

On Windows, use:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File <public-skill-directory>\..\..\shared\gurumaker\scripts\gurumaker.ps1 <arguments>
```

The wrapper accepts `--runtime-status` for a read-only check and
`--runtime-install` for explicit setup. A normal Guru Maker command also
installs the runtime on first use, subject to the host's native network and
filesystem permissions.

The Runtime exposes `gurumaker mode [auto|off] --workspace <project-root>` for
the project-local Guru Mode setting. Reading the mode is non-mutating; changing
it requires an explicit user request under the Guru Mode contract.

The plain `runtime/version` file pins the exact Rust Runtime expected by every
public Skill. The wrapper first accepts an exact
`GURUMAKER_RUNTIME_BIN` override, then the managed version, then an exact
`gurumaker` on `PATH`. If none exists, it downloads the matching release
artifact and `SHA256SUMS`, verifies the digest, and atomically installs the
binary under the user's data directory.

An update never modifies project memory or Guru Pack source slots. A new plugin version
selects a new versioned runtime directory on its next invocation. If setup
fails, report the error and do not claim that Guru Maker memory was read or
changed.
