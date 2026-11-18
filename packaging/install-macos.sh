#!/usr/bin/env bash
set -euo pipefail

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
prefix=${1:?Informe o diretório de instalação}
binary=${2:-}
app_id=dev.veloxtranscode.Velox

fail() {
    printf 'Velox: %s\n' "$*" >&2
    exit 1
}

export PATH="$PATH:/opt/homebrew/bin:/usr/local/bin"
for command in xcrun otool lipo plutil codesign; do
    command -v "$command" >/dev/null || fail "Dependência ausente: $command. Execute xcode-select --install."
done
xcrun --find clang >/dev/null || fail "Instale as ferramentas de linha de comando com xcode-select --install."

for command in ffmpeg ffprobe; do
    if ! tool=$(type -P "$command") || [[ ! -x "$tool" ]]; then
        fail "Dependência ausente: $command. Execute brew install ffmpeg."
    fi
    tool="$(cd -- "$(dirname -- "$tool")" && pwd -P)/${tool##*/}"
    case "$command" in
        ffmpeg) ffmpeg=$tool ;;
        ffprobe) ffprobe=$tool ;;
    esac
done
"$ffprobe" -version >/dev/null || fail "O ffprobe não conseguiu iniciar."
encoders=$("$ffmpeg" -hide_banner -encoders 2>&1) || fail "O FFmpeg não conseguiu listar os encoders."
if ! awk '$1 ~ /^V/ && $2 ~ /^(libx264|prores_ks)$/ { found = 1 } END { exit !found }' <<< "$encoders"; then
    fail "O FFmpeg precisa expor libx264 ou prores_ks."
fi

qt_version() { "$1" -query QT_VERSION 2>/dev/null; }
if [[ -n "${QMAKE:-}" ]]; then
    qmake=$QMAKE
else
    for candidate in qmake6 qmake; do
        if [[ "$(qt_version "$candidate")" == 6.* ]]; then
            qmake=$(type -P "$candidate")
            break
        fi
    done
    if [[ -z "${qmake:-}" ]] && command -v brew >/dev/null; then
        qtbase=$(brew --prefix qtbase)
        qmake="$qtbase/bin/qmake"
    fi
fi
[[ -n "${qmake:-}" && "$(qt_version "$qmake")" == 6.* ]] ||
    fail "Qt 6 ausente. Execute brew install qtbase ou informe o qmake com QMAKE."
for directory in "$("$qmake" -query QT_INSTALL_BINS)" "$("$qmake" -query QT_INSTALL_LIBEXECS)"; do
    if [[ -n "$directory" && -x "$directory/macdeployqt" ]]; then
        deploy="$directory/macdeployqt"
        break
    fi
done
[[ -n "${deploy:-}" ]] || fail "O Qt selecionado precisa incluir macdeployqt."

if [[ -z "$binary" ]]; then
    command -v cargo >/dev/null || fail "Instale o Rust com Cargo para compilar o Velox."
    (
        cd -- "$project"
        export QMAKE="$qmake" QT_INCLUDE_PATH= QT_LIBRARY_PATH=
        cargo build --release --locked --target-dir "$project/target"
    )
    binary="$project/target/release/velox-transcode"
fi
[[ -x "$binary" ]] || fail "Binário de instalação ausente: $binary."
lipo -verify_arch "$(uname -m)" "$binary" || fail "O binário precisa incluir a arquitetura deste Mac."
linked=$(otool -L "$binary")
[[ "$linked" == *QtWidgets.framework/* ]] || fail "O binário deve estar vinculado aos frameworks do Qt 6."
version=$(awk -F '"' '/^version[[:space:]]*=/ { print $2; exit }' "$project/Cargo.toml")
version=${version%%[-+]*}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "Versão inválida no Cargo.toml."

mkdir -p -- "$prefix"
prefix=$(cd -- "$prefix" && pwd -P)
installed="$prefix/Velox Transcode.app"
staging=$(mktemp -d "$prefix/.velox-install.XXXXXX")
lock="$prefix/.velox-install.lock"
bundle="$staging/Velox Transcode.app"
previous="$staging/previous.app"
locked=false
cleanup() {
    if [[ -d "$previous" && ! -e "$installed" ]] && ! mv -- "$previous" "$installed"; then
        printf 'Velox: a instalação anterior está em %s\n' "$previous" >&2
    else
        rm -rf -- "$staging"
    fi
    if $locked; then
        rmdir -- "$lock"
    fi
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
mkdir -- "$lock" || fail "Outro instalador está usando $prefix."
locked=true
if [[ -e "$installed" || -L "$installed" ]]; then
    [[ ! -L "$installed" && -f "$installed/Contents/Info.plist" ]] &&
        [[ "$(plutil -extract CFBundleIdentifier raw -o - "$installed/Contents/Info.plist")" == "$app_id" ]] ||
        fail "Já existe outro item em $installed."
fi

contents="$bundle/Contents"
mkdir -p -- "$contents/MacOS" "$contents/Resources"
install -m755 -- "$binary" "$contents/MacOS/velox-transcode"
install -m644 -- "$project/packaging/icons/macos/Velox.icns" "$contents/Resources/Velox.icns"
for name in host-grotesk instrument-sans; do
    install -m644 -- "$project/web/assets/fonts/$name-OFL.txt" "$contents/Resources/$name-OFL.txt"
done
printf '%s' "$ffmpeg" > "$contents/Resources/ffmpeg.path"
printf '%s' "$ffprobe" > "$contents/Resources/ffprobe.path"
sed "s/@VERSION@/$version/g" "$project/packaging/macos/Info.plist.in" > "$contents/Info.plist"
plutil -lint "$contents/Info.plist"
"$deploy" "$bundle" -no-strip
[[ -d "$contents/Frameworks/QtCore.framework" && -f "$contents/PlugIns/platforms/libqcocoa.dylib" ]] ||
    fail "O bundle ficou sem os frameworks Qt ou o plugin Cocoa."
codesign --verify --deep --strict "$bundle" || fail "O bundle não passou na validação da assinatura."

if [[ -d "$installed" ]]; then
    mv -- "$installed" "$previous"
fi
mv -- "$bundle" "$installed"
printf 'Velox instalado em %s\n' "$installed"
