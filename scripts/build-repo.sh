#!/usr/bin/env bash
# Build the Yuna packages and the local [yuna] pacman repository.
# Runs as a normal user (makepkg refuses root).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/out/repo"

if [[ ${EUID} -eq 0 ]]; then
    echo "error: run build-repo.sh as a normal user, not root" >&2
    exit 1
fi

mkdir -p "$OUT"
rm -f "$OUT"/*.pkg.tar.* "$OUT"/yuna.db* "$OUT"/yuna.files*

for dir in "$ROOT"/packages/*/; do
    echo
    echo "==> building $(basename "$dir")"
    (
        cd "$dir"
        rm -rf pkg src ./*.pkg.tar.*
        makepkg -f --nodeps --noconfirm
        cp ./*.pkg.tar.* "$OUT"/
        rm -f ./*.pkg.tar.*
        rm -rf pkg src
    )
done

if ! compgen -G "$OUT/*.pkg.tar.*" > /dev/null; then
    echo "error: no packages were built" >&2
    exit 1
fi

repo-add --remove --new "$OUT/yuna.db.tar.gz" "$OUT"/*.pkg.tar.*

echo
echo "repository ready: $OUT"
ls -1 "$OUT"
