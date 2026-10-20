#!/bin/sh
# OpenBB Platform MCP server: U.S. and global prices and macro data.
# No key is required for the bundled providers (yfinance, SEC, BLS, OECD, IMF, U.S. Treasury).
set -eu
GIBO_SERVER=openbb
. "$(dirname -- "$0")/lib.sh"

if ! server=$(gibo_first_executable \
    "$GIBO_DATA/openbb/bin/openbb-mcp" \
    "$HOME/.local/bin/openbb-mcp"); then
    gibo_need_setup "openbb-mcp is not installed"
fi

# OpenBB takes ~20 s and ~1 GB to start, so the relay starts it on the first tool
# call and answers the handshake and tool list from a cache written on the first start.
exec python3 -I "$GIBO_MCP_DIR/relay.py" --name openbb --lazy --cache "$GIBO_DATA/openbb/gibo-relay-cache.json" -- \
    "$server" --transport stdio --tool-discovery --enable-cli-tools false
