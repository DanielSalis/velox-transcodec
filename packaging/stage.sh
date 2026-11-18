#!/usr/bin/env bash
set -euo pipefail

project=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
binary=${1:?Informe o binário}
destination=${2:?Informe o diretório de destino}
launcher=${3:-}
app_id=dev.veloxtranscode.Velox

# Exec tem dois níveis de escape segundo a Desktop Entry Specification.
case "$launcher" in
    *$'\n'*|*$'\r'*|*$'\t'*)
        printf 'O caminho do executável contém um caractere de controle.\n' >&2
        exit 1
        ;;
esac
launcher=${launcher//%/%%}
launcher=${launcher//\\/\\\\}
launcher=${launcher//\"/\\\"}
launcher=${launcher//\$/\\\$}
launcher=${launcher//\`/\\\`}
launcher=${launcher//\\/\\\\}

desktop=$(mktemp --suffix=.desktop)
trap 'rm -f -- "$desktop"' EXIT
while IFS= read -r line; do
    if [[ -n "$launcher" && "$line" == Exec=* ]]; then
        printf 'Exec="%s" %%F\n' "$launcher"
    else
        printf '%s\n' "$line"
    fi
done < "$project/packaging/$app_id.desktop" > "$desktop"
desktop-file-validate "$desktop"

install -Dm755 -- "$binary" "$destination/bin/velox-transcode"
install -Dm644 -- "$project/web/assets/velox-transcode.svg" "$destination/share/icons/hicolor/scalable/apps/$app_id.svg"
for name in host-grotesk instrument-sans; do
    install -Dm644 -- "$project/web/assets/fonts/$name-OFL.txt" "$destination/share/licenses/velox-transcode/$name-OFL.txt"
done
install -Dm644 -- "$desktop" "$destination/share/applications/$app_id.desktop"
