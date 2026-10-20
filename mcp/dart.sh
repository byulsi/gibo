#!/bin/sh
# korean-dart-mcp: OpenDART disclosures, financial statements, and shareholdings.
# Key order: the dart_api_key plugin option (DART_API_KEY), then the macOS Keychain
# item opendart-api-key.
set -eu
GIBO_SERVER=dart
. "$(dirname -- "$0")/lib.sh"

if ! gibo_value_set "${DART_API_KEY:-}"; then
    DART_API_KEY=$(gibo_keychain_password opendart-api-key || true)
fi
gibo_value_set "${DART_API_KEY:-}" ||
    gibo_fail "no OpenDART key. Set the dart_api_key option in /plugin, or run: security add-generic-password -a \"\$USER\" -s opendart-api-key -w"
export DART_API_KEY

entry=node_modules/korean-dart-mcp/build/index.js
server=""
for base in "$GIBO_DATA/korean-dart-mcp" "$HOME/.local/share/korean-dart-mcp"; do
    if [ -f "$base/$entry" ]; then
        server="$base/$entry"
        break
    fi
done
[ -n "$server" ] || gibo_need_setup "korean-dart-mcp is not installed"
command -v node >/dev/null 2>&1 || gibo_fail "Node.js 20.19 or later is required"

export GIBO_DART_CONVERTER="${server%/index.js}/lib/dart-xml.js"
exec python3 -I "$GIBO_MCP_DIR/relay.py" --name dart -- node "$server"
