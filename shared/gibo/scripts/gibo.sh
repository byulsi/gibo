#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
SKILL_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
VERSION_FILE="$SKILL_DIR/runtime/version"
RELEASE_BASE_URL=${GIBO_RELEASE_BASE_URL:-https://github.com/byulsi/gibo/releases/download}

fail() {
    printf 'Gibo runtime error: %s\n' "$1" >&2
    exit 1
}

[ -f "$VERSION_FILE" ] || fail "missing runtime version file"
RUNTIME_VERSION=$(tr -d '[:space:]' <"$VERSION_FILE")
[ -n "$RUNTIME_VERSION" ] || fail "runtime version is empty"

detect_target() {
    system=$(uname -s 2>/dev/null || true)
    machine=$(uname -m 2>/dev/null || true)
    case "$system:$machine" in
        Darwin:arm64 | Darwin:aarch64)
            printf '%s\n' "aarch64-apple-darwin"
            ;;
        Darwin:x86_64 | Darwin:amd64)
            printf '%s\n' "x86_64-apple-darwin"
            ;;
        Linux:aarch64 | Linux:arm64)
            printf '%s\n' "aarch64-unknown-linux-gnu"
            ;;
        Linux:x86_64 | Linux:amd64)
            printf '%s\n' "x86_64-unknown-linux-gnu"
            ;;
        *)
            fail "unsupported platform: ${system:-unknown}/${machine:-unknown}"
            ;;
    esac
}

runtime_home() {
    if [ -n "${GIBO_RUNTIME_HOME:-}" ]; then
        printf '%s\n' "$GIBO_RUNTIME_HOME"
    elif [ -n "${XDG_DATA_HOME:-}" ]; then
        printf '%s\n' "$XDG_DATA_HOME/gibo/runtimes"
    elif [ -n "${HOME:-}" ]; then
        printf '%s\n' "$HOME/.local/share/gibo/runtimes"
    else
        fail "cannot resolve runtime home"
    fi
}

version_matches() {
    [ -x "$1" ] || return 1
    output=$("$1" --version 2>/dev/null) || return 1
    [ "$output" = "$RUNTIME_VERSION" ] || [ "$output" = "gibo $RUNTIME_VERSION" ]
}

TARGET=$(detect_target)
RUNTIME_HOME=$(runtime_home)
MANAGED_RUNTIME="$RUNTIME_HOME/$RUNTIME_VERSION/$TARGET/gibo"

find_runtime() {
    if [ -n "${GIBO_RUNTIME_BIN:-}" ]; then
        version_matches "$GIBO_RUNTIME_BIN" ||
            fail "GIBO_RUNTIME_BIN is not Gibo runtime $RUNTIME_VERSION"
        printf '%s\n' "$GIBO_RUNTIME_BIN"
        return 0
    fi

    if version_matches "$MANAGED_RUNTIME"; then
        printf '%s\n' "$MANAGED_RUNTIME"
        return 0
    fi

    path_runtime=$(command -v gibo 2>/dev/null || true)
    if [ -n "$path_runtime" ] && version_matches "$path_runtime"; then
        printf '%s\n' "$path_runtime"
        return 0
    fi
    return 1
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        fail "sha256sum or shasum is required to verify the runtime"
    fi
}

install_runtime() {
    command -v curl >/dev/null 2>&1 || fail "curl is required to install the runtime"

    artifact="gibo-$TARGET"
    release_url="${RELEASE_BASE_URL%/}/v$RUNTIME_VERSION"
    destination_dir=$(dirname -- "$MANAGED_RUNTIME")
    mkdir -p "$destination_dir"

    download_path="$destination_dir/.gibo.download.$$"
    checksums_path="$destination_dir/.SHA256SUMS.$$"
    cleanup() {
        rm -f "$download_path" "$checksums_path"
    }
    trap cleanup EXIT HUP INT TERM

    printf 'Installing Gibo runtime %s for %s...\n' "$RUNTIME_VERSION" "$TARGET" >&2
    curl -fsSL "$release_url/SHA256SUMS" -o "$checksums_path" ||
        fail "could not download SHA256SUMS for v$RUNTIME_VERSION"
    expected=$(
        awk -v artifact="$artifact" '$2 == artifact || $2 == "*" artifact { print $1; exit }' \
            "$checksums_path"
    )
    [ -n "$expected" ] || fail "SHA256SUMS does not contain $artifact"

    curl -fsSL "$release_url/$artifact" -o "$download_path" ||
        fail "could not download $artifact"
    actual=$(sha256_file "$download_path")
    expected=$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')
    actual=$(printf '%s' "$actual" | tr '[:upper:]' '[:lower:]')
    [ "$actual" = "$expected" ] || fail "checksum mismatch for $artifact"

    chmod 755 "$download_path"
    version_matches "$download_path" ||
        fail "downloaded runtime did not report version $RUNTIME_VERSION"
    mv -f "$download_path" "$MANAGED_RUNTIME"
    rm -f "$checksums_path"
    trap - EXIT HUP INT TERM

    printf 'Installed Gibo runtime at %s\n' "$MANAGED_RUNTIME" >&2
    printf '%s\n' "$MANAGED_RUNTIME"
}

case "${1:-}" in
    --runtime-status)
        if runtime=$(find_runtime); then
            printf 'ready\t%s\t%s\n' "$RUNTIME_VERSION" "$runtime"
            exit 0
        fi
        printf 'missing\t%s\t%s\n' "$RUNTIME_VERSION" "$TARGET"
        exit 1
        ;;
    --runtime-install)
        if runtime=$(find_runtime); then
            printf '%s\n' "$runtime"
        else
            install_runtime
        fi
        exit 0
        ;;
esac

if runtime=$(find_runtime); then
    :
else
    runtime=$(install_runtime)
fi
exec "$runtime" "$@"
