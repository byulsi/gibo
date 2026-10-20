#!/bin/sh
# Install the Gibo data servers into the plugin data folder.
# Usage: sh setup.sh [openbb] [dart] [krx]   (no argument installs all three)
# Requires uv (OpenBB, KRX) and Node.js 20.19+ with npm (DART). Versions are pinned.
set -eu

DATA=${CLAUDE_PLUGIN_DATA:-$HOME/.claude/plugins/data/gibo-byulsi}
PYTHON_VERSION=3.12
# The data extensions of openbb==5.0.0, without the openbb meta package, openbb-cli,
# openbb-devtools, openbb-technical, and openbb-charting (indicators are computed by
# the KRX relay). openbb-yfinance is installed without its dependencies because it
# requires technical and charting only for its chart widgets.
OPENBB_PACKAGES="openbb-core[pandas]==2.0.1 openbb-mcp-server==2.0.1 openbb-platform-api==2.0.1
    openbb-bls==2.0.0 openbb-cboe==2.0.0 openbb-cftc==2.0.0 openbb-deribit==2.0.0 openbb-ecb==2.0.0
    openbb-famafrench==2.0.0 openbb-federal-reserve==2.0.0 openbb-finra==2.0.0 openbb-fred==2.0.0
    openbb-government-us==2.0.0 openbb-imf==3.0.0 openbb-jodi==2.0.0 openbb-nasdaq==2.0.0
    openbb-news==2.0.0 openbb-oecd==2.0.0 openbb-sec==2.0.0 openbb-tmx==2.0.0 openbb-us-eia==2.0.0
    yfinance==1.7.0"
OPENBB_NO_DEPS="openbb-yfinance==2.0.0"
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
    # A fresh environment, so packages from an earlier, larger install do not linger.
    rm -rf "$DATA/openbb"
    uv venv --quiet --python "$PYTHON_VERSION" "$DATA/openbb"
    # shellcheck disable=SC2086
    uv pip install --quiet --python "$DATA/openbb/bin/python" $OPENBB_PACKAGES
    # shellcheck disable=SC2086
    uv pip install --quiet --no-deps --python "$DATA/openbb/bin/python" $OPENBB_NO_DEPS
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
    rm -rf "$DATA/korean-dart-mcp"
    mkdir -p "$DATA/korean-dart-mcp"
    cd "$DATA/korean-dart-mcp"
    npm init -y >/dev/null
    npm pkg set "overrides.better-sqlite3=$BETTER_SQLITE3"
    # --omit=optional skips ~520 MB of attachment-conversion extras (onnxruntime,
    # transformers, sharp, canvas, pdfium). Attachment download fails anyway because
    # dart.fss.or.kr refuses programmatic clients.
    npm install --no-fund --no-audit --loglevel=error --omit=optional "$KOREAN_DART_MCP"
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
