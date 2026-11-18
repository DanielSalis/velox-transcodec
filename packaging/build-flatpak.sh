#!/usr/bin/env bash
set -euo pipefail

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
output=${1:?Informe o caminho do bundle Flatpak}
app_id=dev.veloxtranscode.Velox
branch=6.11
cache="${XDG_CACHE_HOME:-$HOME/.cache}/velox-transcode"
mkdir -p -- "$cache"
work=$(mktemp -d "$cache/flatpak.XXXXXX")
trap 'rm -rf -- "$work"' EXIT

source_dir="$work/source"
build_dir="$work/build"
mkdir -p -- "$source_dir/.cargo" "$source_dir/web"
cp -- "$project"/{Cargo.toml,Cargo.lock,build.rs} "$source_dir/"
cp -a -- "$project"/{src,ui} "$source_dir/"
cp -a -- "$project/web/assets" "$source_dir/web/"
(
    cd -- "$source_dir"
    cargo vendor --locked vendor > .cargo/config.toml
)

flatpak build-init "$build_dir" "$app_id" org.kde.Sdk org.kde.Platform "$branch"
flatpak build --bind-mount="/run/velox-source=$source_dir" "$build_dir" bash -euc '
    export PATH="/usr/lib/sdk/rust-stable/bin:$PATH"
    export QMAKE=/usr/bin/qmake
    unset QT_INCLUDE_PATH QT_LIBRARY_PATH
    cd /run/velox-source
    cargo build --release --locked --offline
'
"$project/packaging/stage.sh" "$source_dir/target/release/velox-transcode" "$build_dir/files"
flatpak build-finish \
    --command=velox-transcode \
    --socket=wayland --socket=fallback-x11 --share=ipc --device=dri \
    --filesystem=xdg-videos \
    --talk-name=org.freedesktop.Notifications \
    "$build_dir"
flatpak build-export "$work/repo" "$build_dir" stable
flatpak build-bundle \
    --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
    "$work/repo" "$work/velox-transcode.flatpak" "$app_id" stable
install -m644 -- "$work/velox-transcode.flatpak" "$output"
