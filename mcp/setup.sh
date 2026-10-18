#!/bin/sh
# Install the Gibo data servers into the plugin data folder.
# Usage: sh setup.sh [openbb] [dart] [krx]   (no argument installs all three)
# Requires uv (OpenBB, KRX) and Node.js 20.19+ with npm (DART). Versions are pinned.
set -eu

DATA=${CLAUDE_PLUGIN_DATA:-$HOME/.claude/plugins/data/gibo-byulsi}
PYTHON_VERSION=3.12
OPENBB_PACKAGES="openbb-mcp-server==2.0.1 openbb==5.0.0 openbb-yfinance==2.0.0"
PYKRX_MCP="pykrx-mcp @ git+https://github.com/sharebook-kr/pykrx-mcp@47b92e547f703186b7f9e8c938599ef92fe71cc3"
KOREAN_DART_MCP=korean-dart-mcp@0.10.1
# better-sqlite3 11.x, pinned by korean-dart-mcp, does not build on Node 26.
BETTER_SQLITE3=13.0.3

need() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'Gibo setup: %s is required. %s\n' "$1" "$2" >&2
        exit 1
    }
}

install_openbb() {
    need uv "Install it from https://docs.astral.sh/uv/"
    echo "== OpenBB -> $DATA/openbb"
    uv venv --quiet --allow-existing --python "$PYTHON_VERSION" "$DATA/openbb"
    # shellcheck disable=SC2086
    uv pip install --quiet --python "$DATA/openbb/bin/python" $OPENBB_PACKAGES
    echo "   building the OpenBB extension cache (first start is slow)"
    "$DATA/openbb/bin/python" -c "import openbb" >/dev/null
}

install_krx() {
    need uv "Install it from https://docs.astral.sh/uv/"
    echo "== KRX (pykrx-mcp) -> $DATA/pykrx"
    uv venv --quiet --allow-existing --python "$PYTHON_VERSION" "$DATA/pykrx"
    uv pip install --quiet --python "$DATA/pykrx/bin/python" \
        "$PYKRX_MCP" pykrx==1.2.9 "mcp<2"
}

install_dart() {
    need node "Install Node.js 20.19 or later."
    need npm "npm ships with Node.js."
    echo "== DART (korean-dart-mcp) -> $DATA/korean-dart-mcp"
    mkdir -p "$DATA/korean-dart-mcp"
    cd "$DATA/korean-dart-mcp"
    [ -f package.json ] || npm init -y >/dev/null
    npm pkg set "overrides.better-sqlite3=$BETTER_SQLITE3"
    npm install --no-fund --no-audit --loglevel=error "$KOREAN_DART_MCP"
    node -e "require('better-sqlite3')" ||
        { echo "Gibo setup: better-sqlite3 failed to load" >&2; exit 1; }
    cd - >/dev/null
}

mkdir -p "$DATA"
[ "$#" -gt 0 ] || set -- openbb dart krx
for server in "$@"; do
    case "$server" in
        openbb) install_openbb ;;
        dart) install_dart ;;
        krx) install_krx ;;
        *) printf 'Gibo setup: unknown server %s (expected openbb, dart, krx)\n' "$server" >&2; exit 1 ;;
    esac
done
echo "Gibo data servers installed in $DATA. Restart Claude Code or run /mcp to reconnect."
