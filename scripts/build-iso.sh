#!/usr/bin/env bash
# Build the Yuna OS ISO with mkarchiso. Must run as root.
#
#   sudo pacman -S archiso
#   ./scripts/build-repo.sh
#   sudo ./scripts/build-iso.sh
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$ROOT/out/repo"
STAGE="$ROOT/.iso-profile"
CACHE="$ROOT/.pacman-cache"

die() { echo "error: $*" >&2; exit 1; }

[[ ${EUID} -eq 0 ]] || die "build-iso.sh must run as root (sudo)"
command -v mkarchiso >/dev/null 2>&1 || die "mkarchiso not found — install the archiso package"

[[ -f "$REPO/yuna.db.tar.gz" ]] || die "no package repository at $REPO — run ./scripts/build-repo.sh first"
compgen -G "$REPO/*.pkg.tar.*" >/dev/null || die "repository is empty — run ./scripts/build-repo.sh"

mkdir -p "$CACHE"

echo "==> staging profile"
rm -rf "$STAGE"
cp -a "$ROOT/distro" "$STAGE"

# Point the build-time pacman at the freshly built local repository and at a
# cache directory on this filesystem (the root filesystem may be small).
sed -i "s|@YUNA_REPO@|$REPO|g; s|@YUNA_CACHE@|$CACHE|g" "$STAGE/pacman.conf"

# Ship the repository inside the ISO so the live system can install yuna-*.
mkdir -p "$STAGE/airootfs/opt/yuna/repo"
cp "$REPO"/*.pkg.tar.* "$STAGE/airootfs/opt/yuna/repo/"
repo-add "$STAGE/airootfs/opt/yuna/repo/yuna.db.tar.gz" \
    "$STAGE"/airootfs/opt/yuna/repo/*.pkg.tar.* >/dev/null

echo "==> running mkarchiso"
mkarchiso -v -w "$ROOT/.work" -o "$ROOT/out" "$STAGE"

echo
echo "ISO written to:"
ls -1 "$ROOT"/out/*.iso 2>/dev/null || die "mkarchiso finished but no ISO was found"
