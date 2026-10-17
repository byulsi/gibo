# Hook Guardrails

Gibo bundles two optional lifecycle hooks for Codex and Claude Code.
They reduce common agent mistakes without turning the product into a workflow
engine or adding context to every prompt.

## Behavior

`hooks/hooks.json` listens only to native `Edit`, `Write`, and `apply_patch`
tool calls:

| Event | Scope | Result |
| --- | --- | --- |
| `PreToolUse` | A target under `.gibo/packs/` | Block the direct edit; installed source slots are read-only provenance outside explicit Pack lifecycle operations. |
| `PostToolUse` | A Markdown target under `gibo/` | Run the canonical knowledge validator. Stay silent on success; return compact corrective context on failure. |

The hook does nothing outside an initialized Gibo project. It also does
nothing for unrelated project files, so a normal source-code edit does not
scan investment memory.

## Runtime path

The POSIX and PowerShell launchers find the nearest initialized project and set
its root for the internal `gibo hook` command. They then use the same
version-pinned Runtime wrapper as the public Skills:

```text
native file edit
  -> host plugin hook
  -> hooks/run-hook.sh or hooks/run-hook.ps1
  -> pinned Runtime wrapper
  -> gibo hook pre-tool-use|post-tool-use
```

Path normalization, installed-pack protection, and knowledge validation live
in Rust. Host configuration and launchers must not duplicate those decisions.
The internal hook command is not part of the public CLI contract.

## Activation and limits

Codex requires users to review and trust plugin hooks before they run. Use
`/hooks` to inspect their source and state. Claude Code exposes installed hooks
through its own `/hooks` browser and may be restricted by managed settings.

If hooks are disabled or untrusted, Gibo retrieval, editing, explicit
`knowledge check`, Gibo Packs, and the Decision Graph continue to work through
the public Skills and CLI.

Tool hooks are guardrails, not a security boundary. Shell commands, external
editors, and specialized host tools may mutate files without passing through
these events. Git still owns installed-pack provenance, and explicit
`gibo pack update <wiki|lens|method>` remains the supported source-update
path and never changes local memory.

Gibo intentionally has no session-start, prompt-submit, stop, or
session-end hook. Hooks do not automatically retrieve memory, save
conversations, create records, regenerate graphs, commit changes, or inject
project memory into every session.

## Validation

Run the focused checks after changing hook behavior:

```bash
python3 -m json.tool hooks/hooks.json
sh -n hooks/run-hook.sh
cargo test --all-targets --locked
```

Also run the POSIX launcher smoke and the PowerShell launcher smoke documented
in CI. Review hook discovery in fresh Codex and Claude Code plugin sessions
because host trust and matcher behavior cannot be proven by Rust tests alone.
