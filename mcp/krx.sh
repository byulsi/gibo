#!/bin/sh
# pykrx-mcp: KRX investor flows, short selling, indices, and prices.
# pykrx logs in to data.krx.co.kr with an ID/password account (not Kakao/Naver sign-in).
# Login order: the krx_id/krx_password plugin options (KRX_ID, KRX_PW), then the macOS
# Keychain item krx-data-login (account = ID, password = password).
set -eu
GIBO_SERVER=krx
. "$(dirname -- "$0")/lib.sh"

if ! gibo_value_set "${KRX_ID:-}" || ! gibo_value_set "${KRX_PW:-}"; then
    KRX_ID=$(gibo_keychain_account krx-data-login || true)
    KRX_PW=$(gibo_keychain_password krx-data-login || true)
fi
if ! gibo_value_set "${KRX_ID:-}" || ! gibo_value_set "${KRX_PW:-}"; then
    gibo_fail "no KRX login. Set krx_id and krx_password in /plugin, or run: security add-generic-password -a YOUR_KRX_ID -s krx-data-login -w"
fi
export KRX_ID KRX_PW

if ! server=$(gibo_first_executable \
    "$GIBO_DATA/pykrx/bin/pykrx-mcp" \
    "$HOME/.local/bin/pykrx-mcp"); then
    gibo_need_setup "pykrx-mcp is not installed"
fi

mkdir -p "$GIBO_DATA/matplotlib"
export MPLCONFIGDIR="$GIBO_DATA/matplotlib"
exec "$server"
