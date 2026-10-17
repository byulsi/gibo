#!/bin/sh
set -eu

event=${1:-}
case "$event" in
    pre-tool-use | post-tool-use) ;;
    *)
        printf 'Gibo hook error: unknown event: %s\n' "$event" >&2
        exit 1
        ;;
esac

current=$PWD
while :; do
    if [ -d "$current/gibo" ] &&
        [ -f "$current/.gibo/.gitignore" ]; then
        GIBO_HOOK_WORKSPACE=$current
        export GIBO_HOOK_WORKSPACE
        plugin_root=${CLAUDE_PLUGIN_ROOT:-${PLUGIN_ROOT:-}}
        [ -n "$plugin_root" ] || {
            printf 'Gibo hook error: plugin root is unavailable\n' >&2
            exit 1
        }
        exec sh "$plugin_root/shared/gibo/scripts/gibo.sh" \
            hook "$event"
    fi

    parent=$(dirname -- "$current")
    [ "$parent" != "$current" ] || exit 0
    current=$parent
done
