# Card 5 — Distribuição e instalação

## Fedora Workstation

O instalador compila com o lockfile, verifica Qt 6, Wayland/XKB, FFmpeg e ffprobe, instala o binário, o ícone SVG, o desktop entry e as licenças das fontes em `~/.local`, e atualiza o cache MIME. Use um FFmpeg que ofereça pelo menos um encoder H.264 utilizável, o backend de portal da sessão e o plugin Qt para Wayland. Para compilação use Cargo, um compilador C++ e `qt6-qtbase-devel`. As ferramentas do desktop entry vêm de `desktop-file-utils`. [Dependências Qt e evidências do card 1](card-1-fedora.md)

```sh
./install.sh
~/.local/bin/velox-transcode
```

Se a descoberta dos cabeçalhos Qt falhar com o pacote de desenvolvimento instalado, execute `QMAKE=/usr/bin/qmake6 ./install.sh`. É possível escolher outro diretório com `--prefix` ou instalar um executável release já compilado com `--binary`. O desktop entry registra quatro tipos MIME e aceita vários caminhos locais por meio de `%F`. O usuário pode escolher o Velox na função **Abrir com** do gerenciador de arquivos.

No Fedora 44 testado, `ffmpeg-free` fornecia os executáveis e `libavcodec-freeworld` fornecia `libx264` e NVENC. O RPM resolve os executáveis e as dependências Qt no Fedora. Os encoders utilizáveis dependem dos codecs e dos drivers de cada máquina. O Velox testa cada encoder antes de apresentá-lo na interface. [Resultados de detecção do motor](card-2-media.md)

## Instalação com os pacotes da prévia

Execute o comando do canal escolhido no diretório em que baixou o arquivo.

| Canal | Comando conferido | Ambiente |
| --- | --- | --- |
| Fedora RPM | `sudo dnf install ./velox-transcode-0.1.0-1.fc44.x86_64.rpm` | Fedora 44 x86_64 |
| Flatpak | `flatpak install --user ./velox-transcode.flatpak` | Linux x86_64 com Flatpak e acesso ao runtime KDE 6.11 no Flathub |
| Arch / tarball | `tar -xzf velox-transcode-linux-x86_64.tar.gz && ./velox-transcode/install.sh` | Arch Linux x86_64 atualizado, com `qt6-base`, `qt6-wayland`, `ffmpeg` e `desktop-file-utils` |

Abra o aplicativo pelo menu do desktop. No Flatpak, também é possível executar `flatpak run dev.veloxtranscode.Velox`. O bundle usa FFmpeg e ffprobe do runtime KDE. O sandbox expõe o diretório Vídeos definido pelo sistema, os nós de renderização e as interfaces de janela, portal e notificação. Para arquivos e destinos fora de Vídeos, use os seletores do portal. [Uso do Flatpak](https://docs.flatpak.org/en/latest/using-flatpak.html) · [Permissões do sandbox](https://docs.flatpak.org/en/latest/sandbox-permissions.html)

Para Silverblue, prefira o bundle Flatpak. Para compilar a versão nativa, use um Toolbx com a mesma versão do Fedora e execute `cargo build --release --locked`. De volta ao host, use `./install.sh --binary target/release/velox-transcode` com as bibliotecas Qt, os portais e o FFmpeg do host disponíveis. Este procedimento aguarda validação em uma instalação Silverblue. [Fedora Silverblue e Toolbx](https://docs.fedoraproject.org/en-US/fedora-silverblue/toolbox/)

## Geração e conferência

Em um Fedora 44 x86_64, além dos requisitos de compilação, use `rpm-build`, Python 3.11 ou posterior e Flatpak. Para produzir o bundle, instale o runtime, o SDK e a extensão Rust compatível no mesmo contexto de usuário.

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.kde.Platform//6.11 org.kde.Sdk//6.11 org.freedesktop.Sdk.Extension.rust-stable//25.08
./packaging/release.sh
(cd web/downloads && sha256sum -c SHA256SUMS)
```

`./packaging/release.sh native` gera RPM e tarball, e `./packaging/release.sh flatpak` gera o bundle. O build do Flatpak prepara as dependências Cargo do lockfile no host e compila dentro do SDK sem rede. `VELOX_RELEASE_DIR` permite selecionar o destino. A landing usa `web/downloads/`, um diretório de artefatos gerados e ignorado pelo Git. Execute o build de release antes de servir a landing a partir de um checkout novo e inclua o diretório ao copiar os arquivos do site.

Os três pacotes e o manifesto SHA-256 estão disponíveis nesta cópia de trabalho. A publicação externa depende de autorização, da definição da licença do aplicativo e da confirmação do namespace `dev.veloxtranscode.Velox`. As licenças OFL das fontes integram os três canais.

## Evidências de aceite

| Ambiente | Verificação | Resultado |
| --- | --- | --- |
| Fedora Workstation 44, GNOME/Mutter 50.6, Qt 6.11.2 | `./install.sh` com usuário comum, instalação em `~/.local` e reinstalação em prefixo com espaços | Binário, desktop entry, ícone e cache para MP4, MKV, WebM e MOV presentes. A execução com FFmpeg ausente terminou antes de criar o prefixo. |
| Sessão GNOME Wayland do Fedora | Inicialização do instalado com `WAYLAND_DEBUG=1` | App ID `dev.veloxtranscode.Velox` e entrega de buffers registrados. |
| X11 isolado, Fedora 44 | `desktop-file-validate`, associação MP4 com `gio mime` e abertura por MIME com `gio open`. Lote de dois arquivos aberto via `gio launch` com pasta escolhida pelo portal GTK | `xprop` retornou `velox-transcode` como nome e classe. O app gravou dois MP4, ambos confirmados por ffprobe como H.264 640×360 e AAC. |
| Fedora 44 em contêiner | Instalação do RPM por DNF, cache MIME e inicialização com `QT_QPA_PLATFORM=offscreen` | Instalação e inicialização passaram, com FFmpeg da distribuição e sem GPU exposta ao contêiner. |
| Fedora 44 com GNOME Wayland, Flatpak | Instalação do bundle, validação do desktop exportado, janela Wayland e transcodes no sandbox | O encaminhamento de dois arquivos pelo portal de documentos foi confirmado com ffprobe. A janela apresentou o mesmo app ID Wayland e entregou buffers. O runtime confirmou H.264 por libx264, NVENC na GeForce RTX 3070 Ti e VA-API no nó da Radeon 660M/680M. VA-API no nó NVIDIA falhou no teste. |
| Arch Linux x86_64 em contêiner | Extração e instalação do tarball por usuário comum, validação do desktop entry e inicialização Qt offscreen | Instalação e inicialização passaram com as bibliotecas do Arch. |
| Chromium, landing servida por HTTP e aberta diretamente | 1440 e 390 px, navegação por teclado, mockups, cópia dos três comandos e downloads | Sem overflow horizontal, contraste mínimo medido de 11,70:1 e APCA Lc 77,5. Os bytes baixados por HTTP conferem com o manifesto. |

A correspondência dos identificadores da janela com o desktop entry foi medida em Wayland e X11. A inspeção visual do agrupamento nas docks GNOME e KDE, a sessão gráfica no Arch e os testes em Silverblue ficam pendentes. O teste de notificação no desktop está documentado no card 3. [Evidências da interface nativa](card-3-desktop.md)
