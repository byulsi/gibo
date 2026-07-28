# Guru Maker Installation

Guru Maker is installed through each agent host's plugin system. Codex, Claude
Code, and future hosts reuse five public Skills and one version-pinned Rust
Runtime. The host plugin registers the public Skills; first use resolves their
shared Runtime and project initialization begins with an explicit request.

## Codex

Add the repository marketplace and install the plugin:

```bash
codex plugin marketplace add monarchjuno/guru-maker
codex plugin add guru-maker@monarchjuno
```

Start a new task so the public Skills are discovered. Invoke the fitting Skill
explicitly or make a matching natural-language request:

- `$guru-research` to refresh external evidence using relevant local project
  memory and suitable host-available research skills, tools, or data
  connectors, including when the current evidence is supplied with the
  request;
- `$guru-memory` for memory-only recall, initialization, maintenance of local
  Wiki, Lens, Evidence, and Method records, or an explicit Guru Mode change;
- `$guru-decision` for a requested point-in-time Decision or outcome review,
  the graph, and explicit-action Execution Briefs;
- `$guru-train` for explicit lesson extraction from a reviewed scenario,
  correction, or postmortem; and
- `$guru-packs` only for explicit kind-specific Guru Pack lifecycle work.

Guru Research selects only capabilities that fit the request. It does not
silently install, connect, or configure a missing capability.

Concrete market examples default to familiar U.S. instruments. This is a
documentation convention only; all five Skills remain market- and
asset-neutral.

## Claude Code

Add the repository marketplace and install the plugin:

```text
/plugin marketplace add monarchjuno/guru-maker
/plugin install guru-maker@monarchjuno
/reload-plugins
```

The public Skills are available as `/guru-maker:guru-research`,
`/guru-maker:guru-memory`, `/guru-maker:guru-decision`,
`/guru-maker:guru-train`, and `/guru-maker:guru-packs`. For local development,
load the checkout with:

```bash
claude --plugin-dir .
```

## First use

Each public Skill invokes the bundled shared POSIX or PowerShell wrapper. On
the first actual Guru Maker command, the wrapper:

1. accepts an exact compatible user-provided Runtime when available;
2. otherwise downloads the plugin's pinned target binary and `SHA256SUMS` from
   the matching GitHub Release;
3. verifies SHA-256 and the reported Runtime version;
4. installs it atomically in a versioned user-data directory;
5. runs the requested command.

This occurs under the host's native network and filesystem permissions. The
Runtime is self-contained and installs in a versioned user-data directory.

Project initialization remains explicit. Ask the installed plugin:

```text
$guru-memory Initialize investment memory in this project.
```

or in Claude Code:

```text
/guru-maker:guru-memory Initialize investment memory in this project.
```

Guru Maker then creates `guru-maker/` and `.guru-maker/` in the selected
project. It records Guru Mode `auto` in `.guru-maker/mode`.

`auto` lets the agent make minimum, durable, reviewable memory improvements
during ordinary work in that initialized project. To require an explicit
request for every memory change:

```text
$guru-memory Turn Guru Mode off for this project.
```

The underlying commands are:

```bash
gurumaker mode
gurumaker mode off
gurumaker mode auto
```

Changing the stored mode always requires an explicit request. Auto does not
initialize a project, capture every conversation, or authorize Pack, Git, or
external execution actions.
Guru Mode controls write authorization, not memory eligibility. Personal or
presentation preferences remain outside investment memory unless reviewed as a
repeatable analytical Method with applicability and limits.

## Hook activation

The installed plugin includes `hooks/hooks.json`. It registers only native
file-edit guardrails:

- before an edit, reject direct changes inside `.guru-maker/packs/`;
- after a local `guru-maker/**/*.md` edit, run the canonical knowledge
  validator and return corrective context only when validation fails.

Codex does not trust newly installed plugin hooks automatically. Open `/hooks`,
review the Guru Maker commands, and trust them if you want the guardrails
active. Claude Code also exposes plugin hooks in `/hooks`; managed settings may
disable them.

Hooks remain optional. Without them, the Skills and CLI work normally and the
agent runs `gurumaker knowledge check` explicitly after memory changes. The
hooks do not initialize projects, search or save memory automatically, apply
Guru Mode, add session context, regenerate the graph, or perform Git actions.

The lightweight hook launcher exits without resolving or downloading the
Runtime when the current directory is outside an initialized Guru Maker
project. Inside one, it reuses the same pinned Runtime wrapper as the public
Skills.
See [Hook guardrails](docs/hooks.md) for coverage and limitations.

## Product updates

Update Guru Maker through the current host's plugin marketplace and start a
new task or session. The new shared bundle pins its matching Runtime version.
The next Guru Maker command reuses that exact version if present or installs it
beside older versions.

For Codex:

```bash
codex plugin marketplace upgrade monarchjuno
codex plugin add guru-maker@monarchjuno
```

For Claude Code:

```bash
claude plugin marketplace update monarchjuno
claude plugin update guru-maker@monarchjuno
```

A product update preserves project Markdown, Pack source slots, `PATH`, and
older Runtime versions. Version verification gates each launch to the Runtime
pinned by the installed shared bundle.

See [Runtime distribution and updates](docs/runtime-distribution.md) for the
release and compatibility contract.

## Guru Packs

Install a public or private GitHub repository by asking `$guru-packs` (or its
Claude Code equivalent) or by using the underlying CLI:

```bash
gurumaker pack install https://github.com/owner/repository.git
gurumaker pack install git@github.com:owner/private-repository.git --ref main
```

Private repositories use the machine's existing Git credentials. Pack updates
are separate from product updates, explicit, and require a clean installed
checkout:

```bash
gurumaker pack update wiki
```

Each Pack contains one reusable kind. Installation requires an empty matching
source slot and local collection, stores the exact Git source under
`.guru-maker/packs/<kind>/`, and seeds `guru-maker/<kind>/` once. The source
remains read-only. Search and analysis use only the local copy, which may
evolve independently. Updating or removing the source never changes local
memory.

## Manual Runtime installation

Manual installation is optional. Download an archive from GitHub Releases and
place `gurumaker` (`gurumaker.exe` on Windows) on `PATH`, or install from a
source checkout:

```bash
cargo install --path . --locked
```

The plugin wrapper accepts a `PATH` Runtime only when its version exactly
matches the installed plugin.
