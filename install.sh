#!/bin/sh
# kbinds installer.
#
# One command:
#   wget -qO- https://raw.githubusercontent.com/ys-kalyakin/niri-kbinds/main/install.sh | sh
#
# Overrides:
#   KBINDS_INSTALL_DIR=...   install directory (default ~/.local/bin)
#   KBINDS_VERSION=...       release tag to install (default: latest)
set -eu

REPO="ys-kalyakin/niri-kbinds"
INSTALL_DIR="${KBINDS_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${KBINDS_VERSION:-}"

msg() { printf '%s\n' "$*"; }
err() { printf 'error: %s\n' "$*" >&2; exit 1; }

fetch() { # fetch <url> [outfile] ; "-" prints to stdout
    if command -v curl >/dev/null 2>&1; then
        if [ "${2:--}" = "-" ]; then curl -fsSL "$1"; else curl -fsSL -o "$2" "$1"; fi
    elif command -v wget >/dev/null 2>&1; then
        if [ "${2:--}" = "-" ]; then wget -qO- "$1"; else wget -qO "$2" "$1"; fi
    else
        err "need curl or wget to download files"
    fi
}

[ "$(uname -m)" = "x86_64" ] || err "prebuilt binaries are x86_64-only, this machine is $(uname -m)"
[ "$(id -u)" != "0" ] || msg "warning: running as root — installing to $INSTALL_DIR (set KBINDS_INSTALL_DIR to override)"
command -v tar >/dev/null 2>&1 || err "need tar to unpack the bundle"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [ -z "$VERSION" ]; then
    msg "==> Resolving the latest release"
    VERSION="$(fetch "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name" *: *"\([^"]*\)".*/\1/p' | head -n 1)"
    [ -n "$VERSION" ] || err "could not determine the latest version (GitHub API rate limit or no network?). Retry with: KBINDS_VERSION=<tag>"
fi
case "$VERSION" in v*) ;; *) VERSION="v$VERSION" ;; esac

URL="https://github.com/$REPO/releases/download/$VERSION/niri-kbinds-$VERSION-x86_64-linux-gnu.tar.gz"
msg "==> Downloading kbinds $VERSION"
fetch "$URL" "$TMP/bundle.tar.gz" || err "download failed: $URL"

msg "==> Installing to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
tar xzf "$TMP/bundle.tar.gz" -C "$TMP"
for bin in kbinds kbinds-overlay; do
    [ -f "$TMP/$bin" ] || err "bundle does not contain $bin"
    install -m 0755 "$TMP/$bin" "$INSTALL_DIR/$bin"
done

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) msg "note: $INSTALL_DIR is not in PATH — add it to your shell profile:"
       msg "      export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

msg "==> Installed kbinds $VERSION: $INSTALL_DIR/kbinds, $INSTALL_DIR/kbinds-overlay"
msg "    Wire it up in ~/.config/niri/config.kdl:"
msg "      binds {"
msg "        Mod+Shift+Slash     hotkey-overlay-title=\"Binds: overlay\" { spawn \"kbinds\" \"overlay\"; }"
msg "        Mod+Shift+backslash hotkey-overlay-title=\"Binds: overlay\" { spawn \"kbinds\" \"overlay\"; }"
msg "        Mod+Slash           hotkey-overlay-title=\"Binds: search\"  { spawn \"kbinds\" \"dmenu\"; }"
msg "        Mod+backslash       hotkey-overlay-title=\"Binds: search\"  { spawn \"kbinds\" \"dmenu\"; }"
msg "      }"
msg "    Full docs: https://github.com/$REPO#readme"
