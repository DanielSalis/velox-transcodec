# Velox Transcode para Linux

Extraia o arquivo e execute `./velox-transcode/install.sh`. O binário pré-compilado e os arquivos de integração são instalados em `~/.local`. Para escolher outro destino, use `--prefix`.

O tarball foi produzido no Fedora 44. Ele usa as bibliotecas do sistema e requer Qt 6 com o plugin Wayland, FFmpeg, ffprobe e as ferramentas do pacote `desktop-file-utils`. Para Arch Linux, esses componentes vêm de `qt6-base`, `qt6-wayland`, `ffmpeg` e `desktop-file-utils`. O FFmpeg precisa oferecer um encoder H.264 compatível ou `prores_ks` para Apple ProRes. Execute `~/.local/bin/velox-transcode` ou abra o Velox no menu de aplicativos.
