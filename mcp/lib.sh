# Shared helpers for the Gibo data-server launchers. Sourced, not executed.
# Each launcher sets GIBO_SERVER before sourcing this file.

GIBO_MCP_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
GIBO_DATA=${CLAUDE_PLUGIN_DATA:-$HOME/.claude/plugins/data/gibo-byulsi}

gibo_fail() {
    printf 'Gibo %s: %s\n' "$GIBO_SERVER" "$1" >&2
    exit 1
}

gibo_need_setup() {
    gibo_fail "$1. Install the data servers once with: sh \"$GIBO_MCP_DIR/setup.sh\" $GIBO_SERVER"
}

# A userConfig option that was never filled in can arrive empty or unexpanded.
gibo_value_set() {
    case "${1:-}" in
        "" | '${'*) return 1 ;;
        *) return 0 ;;
    esac
}

# macOS Keychain fallback for credentials stored with `security add-generic-password`.
gibo_keychain_password() {
    command -v security >/dev/null 2>&1 || return 1
    security find-generic-password -s "$1" -w 2>/dev/null
}

gibo_keychain_account() {
    command -v security >/dev/null 2>&1 || return 1
    security find-generic-password -s "$1" 2>/dev/null |
        sed -n 's/^ *"acct"<blob>="\(.*\)"$/\1/p'
}

# Print the first executable path among the arguments.
gibo_first_executable() {
    for candidate in "$@"; do
        if [ -x "$candidate" ]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done
    return 1
}
