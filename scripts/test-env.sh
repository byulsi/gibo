#!/bin/sh
set -eu

usage() {
    printf '%s\n' 'usage: scripts/test-env.sh [--keep]' >&2
    exit 2
}

keep=false
case "${1:-}" in
    '') ;;
    --keep) keep=true ;;
    *) usage ;;
esac

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
WRAPPER="$ROOT/shared/gurumaker/scripts/gurumaker.sh"
VERSION=$(tr -d '[:space:]' <"$ROOT/shared/gurumaker/runtime/version")

fail() {
    printf 'isolated test environment: %s\n' "$1" >&2
    exit 1
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        fail 'sha256sum or shasum is required'
    fi
}

if [ -n "${GURUMAKER_TEST_RUNTIME_BIN:-}" ]; then
    RUNTIME_BIN=$GURUMAKER_TEST_RUNTIME_BIN
else
    cargo build --locked --manifest-path "$ROOT/Cargo.toml" >/dev/null
    RUNTIME_BIN="$ROOT/target/debug/gurumaker"
fi
[ -x "$RUNTIME_BIN" ] || fail "runtime is not executable: $RUNTIME_BIN"
[ "$("$RUNTIME_BIN" --version)" = "$VERSION" ] ||
    fail "runtime does not report pinned version $VERSION"

TEST_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/guru-maker-test.XXXXXX")
cleanup() {
    if [ "$keep" = true ]; then
        printf 'Kept isolated test environment at %s\n' "$TEST_ROOT"
    else
        rm -rf "$TEST_ROOT"
    fi
}
trap cleanup 0 1 2 15

detect_target() {
    case "$(uname -s 2>/dev/null || true):$(uname -m 2>/dev/null || true)" in
        Darwin:arm64 | Darwin:aarch64) printf '%s\n' aarch64-apple-darwin ;;
        Darwin:x86_64 | Darwin:amd64) printf '%s\n' x86_64-apple-darwin ;;
        Linux:aarch64 | Linux:arm64) printf '%s\n' aarch64-unknown-linux-gnu ;;
        Linux:x86_64 | Linux:amd64) printf '%s\n' x86_64-unknown-linux-gnu ;;
        *) fail 'unsupported platform for isolated Runtime test' ;;
    esac
}

TARGET=$(detect_target)
RELEASE_DIR="$TEST_ROOT/releases/download/v$VERSION"
RUNTIME_HOME="$TEST_ROOT/runtime-home"
mkdir -p "$RELEASE_DIR"
ARTIFACT="gurumaker-$TARGET"
cp "$RUNTIME_BIN" "$RELEASE_DIR/$ARTIFACT"
printf '%064d  %s\n' 0 "$ARTIFACT" >"$RELEASE_DIR/SHA256SUMS"

run_wrapper() {
    (
        unset GURUMAKER_RUNTIME_BIN GURUMAKER_RUNTIME_HOME GURUMAKER_RELEASE_BASE_URL
        PATH=/usr/local/bin:/usr/bin:/bin
        export PATH
        GURUMAKER_RUNTIME_HOME="$RUNTIME_HOME" \
        GURUMAKER_RELEASE_BASE_URL="file://$TEST_ROOT/releases/download" \
            sh "$WRAPPER" "$@"
    )
}

INSTALLED_RUNTIME="$RUNTIME_HOME/$VERSION/$TARGET/gurumaker"
if run_wrapper --runtime-install >/dev/null 2>&1; then
    fail 'wrapper accepted a bad Runtime checksum'
fi
[ ! -e "$INSTALLED_RUNTIME" ] ||
    fail 'wrapper left a Runtime behind after a checksum failure'
(
    cd "$RELEASE_DIR"
    printf '%s  %s\n' "$(sha256_file "$ARTIFACT")" "$ARTIFACT" >SHA256SUMS
)
run_wrapper --runtime-install >/dev/null
[ -x "$INSTALLED_RUNTIME" ] || fail 'wrapper did not install into the isolated runtime home'
[ "$(run_wrapper --version)" = "$VERSION" ] ||
    fail 'wrapper did not launch the isolated Runtime'

WORKSPACE_A="$TEST_ROOT/workspace-a"
WORKSPACE_B="$TEST_ROOT/workspace-b"
run_wrapper init "$WORKSPACE_A" >/dev/null
run_wrapper init "$WORKSPACE_B" >/dev/null
for workspace in "$WORKSPACE_A" "$WORKSPACE_B"; do
    [ -d "$workspace/guru-maker" ] || fail "workspace was not initialized: $workspace"
    [ -f "$workspace/.guru-maker/.gitignore" ] ||
        fail "workspace marker is missing: $workspace"
    [ "$(run_wrapper mode --workspace "$workspace")" = "Guru Mode: auto" ] ||
        fail "workspace did not default to Guru Mode auto: $workspace"
done
run_wrapper mode off --workspace "$WORKSPACE_A" >/dev/null
[ "$(run_wrapper mode --workspace "$WORKSPACE_A")" = "Guru Mode: off" ] ||
    fail "workspace did not retain Guru Mode off: $WORKSPACE_A"
[ "$(run_wrapper mode --workspace "$WORKSPACE_B")" = "Guru Mode: auto" ] ||
    fail "Guru Mode leaked across workspaces"
for workspace in "$WORKSPACE_A" "$WORKSPACE_B"; do
    [ -d "$workspace/.guru-maker/packs" ] ||
        fail "workspace pack directory is missing: $workspace"
    [ -z "$(find "$workspace/.guru-maker/packs" -mindepth 1 -print -quit)" ] ||
        fail "workspace initialization created pack content: $workspace"
done

WIKI_A="$WORKSPACE_B/guru-maker/wiki/logical-qubits.md"
WIKI_B="$WORKSPACE_B/guru-maker/wiki/surface-code.md"
printf '%s\n' \
    '---' \
    'id: wiki:logical-qubits' \
    'title: Logical qubits' \
    'summary: Explains the error-corrected unit used by quantum algorithms.' \
    'as_of: 2026-07-30T09:00:00+09:00' \
    'aliases: [logical qubit]' \
    'tags: [quantum-computing]' \
    'see_also: [wiki:surface-code]' \
    '---' \
    '' \
    '# Error correction' \
    '' \
    'A logical qubit is encoded across physical qubits.' >"$WIKI_A"
printf '%s\n' \
    '---' \
    'id: wiki:surface-code' \
    'title: Surface code' \
    'summary: Describes a family of quantum error-correction schemes.' \
    'as_of: 2026-07-30T09:00:00+09:00' \
    'tags: [quantum-computing]' \
    '---' \
    '' \
    '# Mechanism' \
    '' \
    'Local checks detect errors without measuring the encoded state.' >"$WIKI_B"

run_wrapper knowledge search 'logical qubits' \
    --kind wiki --kind method --candidates --limit 6 \
    --workspace "$WORKSPACE_B" --json |
    python3 -c '
import json, sys
items = json.load(sys.stdin)
assert items and items[0]["id"] == "wiki:logical-qubits"
assert "text" not in items[0]
assert items[0]["match_tier"] == "exact_metadata"
'
run_wrapper knowledge neighbors wiki:logical-qubits \
    --workspace "$WORKSPACE_B" --json |
    python3 -c '
import json, sys
result = json.load(sys.stdin)
assert [item["id"] for item in result["outgoing"]] == ["wiki:surface-code"]
assert result["incoming"] == []
'
run_wrapper knowledge neighbors wiki:surface-code \
    --workspace "$WORKSPACE_B" --json |
    python3 -c '
import json, sys
result = json.load(sys.stdin)
assert [item["id"] for item in result["incoming"]] == ["wiki:logical-qubits"]
'
run_wrapper knowledge health --kind wiki --workspace "$WORKSPACE_B" --json |
    python3 -c '
import json, sys
health = json.load(sys.stdin)["kinds"]
assert len(health) == 1
assert health[0]["kind"] == "wiki"
assert health[0]["documents"] == 2
'
run_wrapper knowledge check --workspace "$WORKSPACE_B" >/dev/null

printf 'Isolated Runtime test passed (%s)\n' "$TEST_ROOT"
