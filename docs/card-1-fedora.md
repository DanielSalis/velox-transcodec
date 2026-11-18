# Card 1 — Fundação Rust no Fedora

A janela de prova aceitava caminhos de uma transferência nativa, do seletor múltiplo ou da linha de comando. Ela listava os caminhos recebidos para conferir a entrada. A interface atual está documentada no [card 3](card-3-desktop.md).

## Compilar e executar

O manifesto usa Rust Edition 2024 e declara Rust 1.95 como mínimo do Slint. O build e as verificações foram executados com Rust/Cargo 1.99.1 no Fedora Workstation 44 x86_64.

Para uma estação de desenvolvimento Fedora, as dependências da prova são:

```sh
sudo dnf install cargo rustfmt clippy gcc-c++ qt6-qtbase-devel qt6-qtwayland xdg-desktop-portal
cargo build --locked
cargo run --locked -- "/caminho/um vídeo.mp4" "/caminho/dois.mkv"
```

Use o provedor do FileChooser da sessão, GNOME/GTK ou KDE, em conjunto com `xdg-desktop-portal`. O QPA `wayland` vem de `qt6-qtwayland` e o QPA `xcb` de `qt6-qtbase-gui`. O Fedora 44 verificado tinha os runtimes `libxkbcommon` e `libxkbcommon-x11` e o pacote `wayland-devel` 1.26.0. Os cabeçalhos `libxkbcommon-devel` e `libxkbcommon-x11-devel` não foram necessários para o build do backend Qt. FFmpeg/ffprobe tornam-se dependências funcionais no card 2. [Documentação do backend Qt](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_qt/) · [Backends do seletor rfd](https://docs.rs/rfd/0.18.0/rfd/)

No Silverblue, instale as dependências de build em um Toolbx compatível com a versão Fedora do host e compile nele. Se executar o binário no host, confira as bibliotecas Qt e os plugins QPA no host. Portal e backend do FileChooser pertencem à sessão gráfica. O caminho em Toolbx e a instalação final no Silverblue ficam para a validação do card 5. [Fedora Silverblue e Toolbx](https://docs.fedoraproject.org/en-US/fedora-silverblue/toolbox/)

O teste local usou os cabeçalhos e links de Qt extraídos do RPM `qt6-qtbase-devel` em um diretório temporário, com `QT_INCLUDE_PATH` e `QT_LIBRARY_PATH`. O binário usou as bibliotecas de runtime já presentes na máquina. Rustfmt e Clippy foram executados a partir dos RPMs Fedora correspondentes ao Rust instalado.

## Evidências

| Ambiente | Verificação |
| --- | --- |
| Fedora Workstation 44, GNOME Shell/Mutter 50.6, Qt 6.11.2 | A janela abriu via QPA Wayland. O trace registrou `xdg_toplevel.set_app_id("dev.veloxtranscode.Velox")`, configuração e entrega de buffers Wayland. |
| Fedora Workstation 44, Weston 15.0.1 aninhado em Xvfb, QPA Wayland | Aplicativo e origem GTK conectados ao socket Wayland, com `DISPLAY` removido. O trace registrou `text/uri-list` e `wl_data_device.drop`. Os dois arquivos apareceram na janela, inclusive `um vídeo.mp4`. |
| Fedora Workstation 44, Xvfb 21.1.25, QPA xcb | Arraste de uma origem GTK entregou os dois arquivos. `xprop` registrou `WM_CLASS(STRING) = "velox-transcode", "velox-transcode"`. |
| Xvfb com D-Bus de teste, XDG Desktop Portal 1.22.1 e GTK 1.15.3 | A chamada `OpenFile` teve `multiple=true`. Cancelar liberou o botão para nova tentativa. Selecionar ambos retornou duas URIs com código de resposta zero e acrescentou os dois caminhos à janela. |

Passaram `cargo build --locked`, `cargo fmt --all --check`, `cargo clippy --locked --all-targets -- -D warnings` e `cargo test --locked`. Os três testes cobrem o intervalo de qualidade, preservação de múltiplos caminhos e rejeição de texto simples como entrada de arraste.

O teste de arraste usou uma origem GTK controlada. A matriz com Nautilus em GNOME, Dolphin em KDE e agrupamento na dock fica para o card 5. O seletor da prova já exercita o portal, com `parent_window` vazio. O agrupamento do .desktop depende da aplicação empacotada.

## Entrega aos cards 2 e 3

- **Motor.** Implemente probe, detecção e fila usando [os contratos Rust](../src/domain/mod.rs). Os eventos têm `MediaId` e `BatchId` para correlação. Preserve a escala de progresso de 0 a 100 e envie o tamanho final só em `JobOutput`.
- **Desktop.** O card 3 conservou o ID em [src/lib.rs](../src/lib.rs) e integrou a recepção por `DropArea` e o seletor assíncrono à fila.
