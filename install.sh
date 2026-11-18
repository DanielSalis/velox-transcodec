#!/usr/bin/env bash
set -euo pipefail

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
system=$(uname -s)
case "$system" in
    Linux) prefix="$HOME/.local" ;;
    Darwin) prefix="$HOME/Applications" ;;
    *) prefix= ;;
esac
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
            printf 'Destino padrão no Fedora: ~/.local. No macOS: ~/Applications.\n'
            exit 0
            ;;
        *) fail "Opção desconhecida: $1." ;;
    esac
done

case "$system" in
    Darwin)
        [[ -f "$project/packaging/install-macos.sh" ]] || fail "Use o repositório completo para instalar no macOS."
        exec bash "$project/packaging/install-macos.sh" "$prefix" "$binary"
        ;;
    Linux) ;;
    *) fail "Sistema sem instalador: $system." ;;
esac

for command in ffmpeg ffprobe desktop-file-validate update-desktop-database ldd ldconfig; do
    command -v "$command" >/dev/null || fail "Dependência ausente: $command. Consulte o README para as dependências."
done
ffprobe -version >/dev/null || fail "O ffprobe não conseguiu iniciar."
encoders=$(ffmpeg -hide_banner -encoders 2>&1) || fail "O FFmpeg não conseguiu listar os encoders."
if ! awk '$1 ~ /^V/ && $2 ~ /^(libx264|h264_vaapi|h264_nvenc|prores_ks)$/ { found = 1 } END { exit !found }' <<< "$encoders"; then
    fail "O FFmpeg precisa expor um encoder H.264 do Velox ou prores_ks."
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
staging=$(mktemp -d "$prefix/.velox-install.XXXXXX")
trap 'rm -rf -- "$staging"' EXIT
"$project/packaging/stage.sh" "$binary" "$staging" "$prefix/bin/velox-transcode"
mapfile -d '' files < <(find "$staging" -type f -print0)
for file in "${files[@]}"; do
    destination="$prefix/${file#"$staging"/}"
    mkdir -p -- "${destination%/*}"
    [[ ! -d "$destination" ]] || fail "O destino é um diretório: $destination."
done
for file in "${files[@]}"; do
    mv -fT -- "$file" "$prefix/${file#"$staging"/}"
done
update-desktop-database "$prefix/share/applications"
if command -v gtk-update-icon-cache >/dev/null; then
    gtk-update-icon-cache -f -t "$prefix/share/icons/hicolor" ||
        printf 'Velox: os ícones foram instalados, mas o cache GTK não pôde ser atualizado.\n' >&2
fi
printf 'Velox instalado em %s/bin/velox-transcode\n' "$prefix"
