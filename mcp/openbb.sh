#!/bin/sh
# OpenBB Platform MCP server: U.S. and global prices, macro data, and technical calculations.
# No key is required for the bundled providers (yfinance, SEC, BLS, OECD, IMF, U.S. Treasury).
set -eu
GIBO_SERVER=openbb
. "$(dirname -- "$0")/lib.sh"

if ! server=$(gibo_first_executable \
    "$GIBO_DATA/openbb/bin/openbb-mcp" \
    "$HOME/.local/bin/openbb-mcp"); then
    gibo_need_setup "openbb-mcp is not installed"
fi

# The relay hides the 2026-07-28 discovery probe that FastMCP 4 servers reject
# when Claude Code sends it together with the initialize handshake.
exec python3 -I "$GIBO_MCP_DIR/mcp-legacy-proxy.py" "$server" \
    --transport stdio --tool-discovery --enable-cli-tools false
