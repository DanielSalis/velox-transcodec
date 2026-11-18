#!/usr/bin/env bash
set -euo pipefail

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
prefix="$HOME/.local"
binary=

fail() {
    printf 'Velox: %s\n' "$*" >&2
    exit 1
}

while (( $# > 0 )); do
    case "$1" in
        --prefix|--binary)
            (( $# >= 2 )) && [[ -n "$2" ]] || fail "Informe o caminho para $1."
            if [[ "$1" == --prefix ]]; then
                prefix=$2
            else
                binary=$2
            fi
            shift 2
            ;;
        -h|--help)
            printf 'Uso: %s [--prefix DIRETÓRIO] [--binary BINÁRIO]\n' "$0"
            exit 0
            ;;
        *) fail "Opção desconhecida: $1." ;;
    esac
done

for command in ffmpeg ffprobe desktop-file-validate update-desktop-database ldd ldconfig; do
    command -v "$command" >/dev/null || fail "Dependência ausente: $command. Consulte o README para as dependências."
done
ffprobe -version >/dev/null || fail "O ffprobe não conseguiu iniciar."
encoders=$(ffmpeg -hide_banner -encoders 2>&1) || fail "O FFmpeg não conseguiu listar os encoders."
if ! awk '$1 ~ /^V/ && $2 ~ /^(libx264|h264_vaapi|h264_nvenc)$/ { found = 1 } END { exit !found }' <<< "$encoders"; then
    fail "O FFmpeg precisa expor libx264, h264_vaapi ou h264_nvenc."
fi

libraries=$(ldconfig -p)
for library in libwayland-client.so.0 libxkbcommon.so.0 libxkbcommon-x11.so.0; do
    if ! awk -v name="$library" '$1 == name { found = 1 } END { exit !found }' <<< "$libraries"; then
        fail "Biblioteca ausente: $library. No Fedora, confira wayland-libs, libxkbcommon e libxkbcommon-x11."
    fi
done

if [[ -z "$binary" && -x "$project/bin/velox-transcode" && ! -f "$project/Cargo.toml" ]]; then
    binary="$project/bin/velox-transcode"
fi
if [[ -z "$binary" ]]; then
    command -v cargo >/dev/null || fail "Cargo é necessário para compilar a partir do código."
    if command -v pkg-config >/dev/null; then
        if ! pkg-config --exists wayland-client xkbcommon; then
            printf 'Cabeçalhos Wayland/XKB opcionais neste backend Qt. No Fedora: wayland-devel e libxkbcommon-devel.\n'
        fi
    fi
    (cd -- "$project" && cargo build --release --locked --target-dir "$project/target")
    binary="$project/target/release/velox-transcode"
fi
[[ -x "$binary" ]] || fail "Binário de instalação ausente: $binary."
linked=$(ldd "$binary") || fail "Não foi possível ler as dependências do binário."
[[ "$linked" != *'not found'* ]] || fail "Bibliotecas em falta no binário: $linked"
[[ "$linked" == *libQt6Widgets.so.6* ]] || fail "O binário foi compilado sem Qt 6. Instale qt6-qtbase-devel e rode com QMAKE=/usr/bin/qmake6."
if command -v qtpaths6 >/dev/null; then
    plugins=$(qtpaths6 --query QT_INSTALL_PLUGINS)
    [[ -f "$plugins/platforms/libqwayland.so" ]] || fail "Plugin Qt para Wayland ausente. No Fedora: qt6-qtwayland."
fi

mkdir -p -- "$prefix"
prefix=$(cd -- "$prefix" && pwd -P)
"$project/packaging/stage.sh" "$binary" "$prefix" "$prefix/bin/velox-transcode"
update-desktop-database "$prefix/share/applications"
printf 'Velox instalado em %s/bin/velox-transcode\n' "$prefix"
