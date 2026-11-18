#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
mode=${1:-all}
case "$mode" in
    all|native|flatpak) ;;
    *) printf 'Uso: %s [all|native|flatpak]\n' "$0" >&2; exit 2 ;;
esac

output=${VELOX_RELEASE_DIR:-"$project/web/downloads"}
mkdir -p -- "$output"
output=$(cd -- "$output" && pwd -P)
version=$(python3 -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["package"]["version"])' "$project/Cargo.toml")
arch=$(uname -m)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

checksums() {
    (
        cd -- "$output"
        sha256sum -- velox-transcode*.rpm velox-transcode*.tar.gz velox-transcode*.flatpak > "$work/SHA256SUMS"
        install -m644 -- "$work/SHA256SUMS" SHA256SUMS
    )
}

if [[ "$mode" != flatpak ]]; then
    for command in cargo rpmbuild desktop-file-validate; do
        command -v "$command" >/dev/null || { printf 'Dependência ausente: %s\n' "$command" >&2; exit 1; }
    done
    (cd -- "$project" && cargo build --release --locked --target-dir "$project/target")
    binary="$project/target/release/velox-transcode"
    linked=$(ldd "$binary")
    if [[ "$linked" != *libQt6Widgets.so.6* || "$linked" == *'not found'* ]]; then
        printf 'O binário precisa estar vinculado ao Qt 6. Confira qt6-qtbase-devel e o QMAKE.\n' >&2
        exit 1
    fi
    rpm_top="$work/rpm"
    mkdir -p -- "$rpm_top/SOURCES"
    "$project/packaging/stage.sh" "$binary" "$work/rpm-payload/usr"
    tar -czf "$rpm_top/SOURCES/velox-rpm-payload.tar.gz" -C "$work/rpm-payload" usr
    rpmbuild -bb --define "_topdir $rpm_top" --define "velox_version $version" \
        --define 'debug_package %{nil}' --define '_build_id_links none' "$project/packaging/velox-transcode.spec"
    install -m644 -- "$rpm_top"/RPMS/"$arch"/*.rpm "$output/"

    portable="$work/velox-transcode"
    install -Dm755 -- "$binary" "$portable/bin/velox-transcode"
    install -Dm755 -- "$project/install.sh" "$portable/install.sh"
    install -Dm755 -- "$project/packaging/stage.sh" "$portable/packaging/stage.sh"
    install -Dm644 -- "$project/packaging/dev.veloxtranscode.Velox.desktop" "$portable/packaging/dev.veloxtranscode.Velox.desktop"
    install -Dm644 -- "$project/web/assets/velox-transcode.svg" "$portable/web/assets/velox-transcode.svg"
    for name in host-grotesk instrument-sans; do
        install -Dm644 -- "$project/web/assets/fonts/$name-OFL.txt" "$portable/web/assets/fonts/$name-OFL.txt"
    done
    install -Dm644 -- "$project/packaging/TARBALL.md" "$portable/README.md"
    tar -czf "$work/velox-transcode-linux-$arch.tar.gz" -C "$work" velox-transcode
    install -m644 -- "$work/velox-transcode-linux-$arch.tar.gz" "$output/"
    checksums
fi

if [[ "$mode" != native ]]; then
    "$project/packaging/build-flatpak.sh" "$output/velox-transcode.flatpak"
    checksums
fi
printf 'Release local em %s\n' "$output"
