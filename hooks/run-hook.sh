#!/bin/sh
set -eu

event=${1:-}
case "$event" in
    pre-tool-use | post-tool-use) ;;
    *)
        printf 'Guru Maker hook error: unknown event: %s\n' "$event" >&2
        exit 1
        ;;
esac

current=$PWD
while :; do
    if [ -d "$current/guru-maker" ] &&
        [ -f "$current/.guru-maker/.gitignore" ]; then
        GURUMAKER_HOOK_WORKSPACE=$current
        export GURUMAKER_HOOK_WORKSPACE
        plugin_root=${CLAUDE_PLUGIN_ROOT:-${PLUGIN_ROOT:-}}
        [ -n "$plugin_root" ] || {
            printf 'Guru Maker hook error: plugin root is unavailable\n' >&2
            exit 1
        }
        exec sh "$plugin_root/shared/gurumaker/scripts/gurumaker.sh" \
            hook "$event"
    fi

    parent=$(dirname -- "$current")
    [ "$parent" != "$current" ] || exit 0
    current=$parent
done
