# Card 9 — Instalação no Fedora e macOS

`./install.sh` detecta o sistema, verifica as dependências e usa `cargo build --release --locked`. `--binary` aceita um executável release da arquitetura local. `--prefix` seleciona a raiz de instalação no Fedora ou a pasta de aplicativos no macOS. Em ambos os sistemas o instalador prepara os arquivos antes de colocá-los no destino.

## Fedora

A instalação do usuário grava em `~/.local`. O desktop entry `dev.veloxtranscode.Velox.desktop` aceita múltiplos vídeos e registra MP4, MOV, MKV e WebM. As instalações local, RPM, tarball e Flatpak usam o SVG e os PNG em 16, 24, 32, 48, 64, 128 e 256 px no tema `hicolor`. O script local atualiza o cache MIME e, quando disponível, o cache de ícones do GTK. [Dependências do Fedora e procedimento Flatpak para Silverblue](card-5-distribuicao.md)

No GNOME, abra **Velox Transcode** pelo menu de aplicativos e use **Adicionar aos favoritos** para fixar o ícone. A associação procura `dev.veloxtranscode.Velox` em Wayland e `velox-transcode` em X11. [Desktop entry](../packaging/dev.veloxtranscode.Velox.desktop)

## macOS

Com as ferramentas de linha de comando da Apple e Rust com Cargo instalados, execute na raiz do repositório.

```bash
brew install qtbase ffmpeg
./install.sh
open "$HOME/Applications/Velox Transcode.app"
```

O instalador usa o Qt 6 selecionado pelo `qmake`. Defina `QMAKE` com o caminho do Qt desejado caso haja mais de uma instalação. `macdeployqt` incorpora os frameworks e plugins, incluindo Cocoa, e aplica a assinatura local ad hoc. O script valida o `Info.plist` e a assinatura antes de trocar a versão instalada. [Qt para macOS](https://doc.qt.io/qt-6/macos-deployment.html) · [Qt Base no Homebrew](https://formulae.brew.sh/formula/qtbase)

O bundle usa `dev.veloxtranscode.Velox` e `Velox.icns` com versões Retina. O Finder encaminha os vídeos de **Abrir com** para a fila. O seletor usa AppKit, e a notificação de conclusão usa a permissão do Notification Center. Para manter o ícone depois de fechar o Velox, use **Opções → Manter no Dock**. O instalador registra onde estão FFmpeg e ffprobe. Rode-o novamente se esses executáveis mudarem de caminho. [Metadados do bundle](../packaging/macos/Info.plist.in) · [Evento de abertura do Qt](https://doc.qt.io/qt-6/qfileopenevent.html)

## Verificação

| Ambiente | Resultado |
| --- | --- |
| Fedora Workstation 44 x86_64 | Compilação release, instalação como usuário em prefixo com espaços, reinstalação com `--binary` e saída antecipada com FFmpeg ausente. O desktop entry passou no `desktop-file-validate` e os quatro tipos MIME aparecem no cache. |
| GNOME Shell 50.6 no Fedora | Em sessão isolada com monitor virtual, o Dash reconheceu o atalho, fixou o ícone e abriu o Velox. O GNOME associou a janela ao desktop entry e manteve um único ícone ao minimizar, restaurar, fechar e reabrir. |
| Wayland / Weston e X11 / Openbox no Fedora | O instalado desenhou a janela com o app ID esperado em Wayland. No X11, o GIO abriu um MP4 por MIME, a janela exibiu o ícone e foi restaurada após minimizar. O seletor GTK escolheu o destino e o lote gerou um MP4 H.264 High conferido com `ffprobe`. O monitor D-Bus registrou a chamada de conclusão com o app ID. A sessão isolada estava sem serviço de notificações. |
| RPM, Flatpak e tarball x86_64 | RPM e Flatpak novos contêm os tamanhos de ícone previstos. O Flatpak foi reinstalado no contexto de teste do usuário e o tarball foi instalado em um prefixo com espaços. |
| Ícones | O GTK resolveu os sete PNG instalados nas dimensões esperadas. O ICNS contém dez entradas. O símbolo foi revisado em tamanhos pequenos sobre fundos escuro e claro. |
| Rust e Qt | `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` e `cargo test --locked --test media_pipeline -- --ignored` passaram. Um teste com Qt no Fedora verificou a ponte de abertura de arquivos e o cálculo de redimensionamento usado no macOS. |
| macOS | O teste simulado em Linux verificou o layout, o plist, os caminhos e a reinstalação, incluindo preservação do bundle anterior quando a assinatura é rejeitada. Compilação, assinatura e execução reais, seletor, Finder, Notification Center e Dock aguardam teste em Apple Silicon. |
| Silverblue e KDE | A instalação em Silverblue e a verificação do painel do KDE aguardam sessões nesses ambientes. |
