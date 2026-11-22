# Arquitetura

## Fronteiras e propriedade

| Caminho previsto | Responsabilidade | Card |
| --- | --- | --- |
| `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/lib.rs`, `src/domain/` | Pacote binário Rust Edition 2024, contratos e integração de módulos | 1 |
| `src/media/`, `src/queue/`, `tests/media/` | ffprobe, detecção de encoder, parâmetros de FFmpeg, execução e progresso | 2 |
| `src/main.rs`, `src/desktop/`, `ui/` | Janela Slint, entrada de arquivos, UI e ponte de comando/evento | 3 |
| `web/` | Site estático e mockups interativos, com o SVG da marca também usado pela instalação | 4 |
| `install.sh`, `packaging/` | Instalação do usuário, desktop entry, formatos de distribuição e verificação | 5 |

O card 1 entregou o manifesto, o lockfile, o perfil release e a prova de entrada. A janela do produto começa em [`src/main.rs`](../src/main.rs), com a integração em [`src/desktop`](../src/desktop) e o layout em [`ui/desktop.slint`](../ui/desktop.slint).

## Desktop no Fedora e macOS

**Slint 1.19.0 com estilo próprio e backend Qt 6.** O backend Winit do Slint 1.19.0 usa a série 0.30 e não expõe arraste de arquivos externo em Wayland nativo. O backend Qt encaminha eventos nativos para `DropArea` e popula `DataTransfer::file_paths()` a partir dos arquivos oferecidos. A Qt é uma dependência nativa em runtime. O backend de janela permanece independente da aceleração do encoder de vídeo. [Backend Qt do Slint 1.19.0](https://docs.rs/crate/i-slint-backend-qt/1.19.0/source/qt_window.rs) · [Arraste em Wayland no Winit](https://github.com/rust-windowing/winit/pull/4571)

Os seletores de vídeos e de pasta de destino usam `rfd` assíncrono com XDG Desktop Portal no Fedora e AppKit no macOS. Arraste, seleção, argumentos de abertura e eventos do Finder convergem em `QueueCommand::AddFiles`. Os valores permanecem `PathBuf` até a apresentação. Os seletores abrem sem uma janela pai exportada pelo backend Qt. [rfd 0.18.0](https://docs.rs/rfd/0.18.0/rfd/) · [Qt QFileOpenEvent](https://doc.qt.io/qt-6/qfileopenevent.html)

O desktop entry, o ícone e o app ID Wayland usam `dev.veloxtranscode.Velox`, sujeito à confirmação do namespace para publicação. Em X11, `StartupWMClass=velox-transcode` corresponde à classe medida com o aplicativo instalado. [Ambientes e evidências](card-5-distribuicao.md)

A barra de título é desenhada em Slint. O backend Qt aplica os estados da janela e executa a movimentação. O redimensionamento parte de áreas Slint nas bordas e nos cantos e delega ao compositor por `QWindow::startSystemResize`. No macOS, a ponte Qt ajusta a geometria durante o arraste se o backend Cocoa recusar o redimensionamento nativo. A ação Fechar usa o mesmo pedido de fechamento do sistema para aguardar a limpeza de um lote ativo. [Barra de título e verificações](card-6-barra-de-titulo.md)

## Contratos de integração

As interfaces estão em [`src/domain`](../src/domain). Elas são independentes de Slint e FFmpeg.

| Contrato | Campos e comportamento |
| --- | --- |
| `MediaInfo` | `MediaId` da sessão, caminho, stream de vídeo e streams de áudio, duração/bitrate opcionais e bytes originais. FPS racional com numerador e denominador positivos quando disponível |
| `EncoderCapability` | `EncoderTarget` com nó VA-API, índice NVENC ou x264. `support` descreve os controles validados ou o motivo tipado de falha |
| `OutputCapabilities` | Encoders e controles H.264 validados, mais os perfis ProRes disponíveis ou o motivo de indisponibilidade |
| `OutputSettings` | Pasta de destino e saída tipada. H.264 recebe encoder, qualidade, preset, perfil, áudio e contêiner. ProRes recebe perfil e áudio e usa MOV |
| `QueueCommand` | Adicionar arquivos, sondar encoders, iniciar uma seleção com configurações do lote e cancelar por `BatchId` |
| `QueueEvent` | Entrada, probe, capacidades, estado, progresso e conclusão. Erros tipados distinguem rejeição de comando e falha de trabalho |
| `ProgressSnapshot`, `BatchProgress` e `JobOutput` | Tempo processado, FPS, velocidade, ETA e percentual. Percentuais usam a escala 0–100. A saída inclui destino, bytes e redução a partir do tamanho final |

A interface recebe `QueueEvent` de [`queue::spawn`](../src/queue/mod.rs) e aplica os eventos num único consumidor no event loop do Slint. O modelo da tabela atualiza por ID de mídia e a UI desabilita mudanças no lote ativo. Tokio executa probes, inventário de hardware e transcodes. Canais limitados e coalescência de progresso preservam a responsividade. `ItemAdded` inicia a sondagem e `MediaProbed` sinaliza que o item está pronto. Falha de probe, fila, execução e estados terminais chegam em `ItemChanged`. `BatchFinished` fornece o resumo a `notify-rust`, que usa D-Bus no Fedora e o Notification Center no macOS. A fila termina mesmo quando o serviço de notificações está indisponível.

## Estratégia de mídia

- Invocar ffprobe com saída JSON e selecionar a primeira trilha de vídeo útil, excluindo capas anexas. Tratar `N/A`, taxa indefinida e ausência de duração ou bitrate como valores opcionais. Exibir resumo do stream e duração do arquivo quando disponível. [FFprobe](https://ffmpeg.org/ffprobe.html)
- Cruzar `ffmpeg -encoders` com um encode curto por encoder e por nó VA-API acessível. Enumerar nós `/dev/dri/renderD*`, registrar diagnóstico de inicialização e filtrar opções inviáveis. Ordem inicial determinística VA-API, NVENC, libx264, ajustável na UI. Se libx264 não iniciar, apresentar diagnóstico de build e dependências.
- Os valores 18–28 têm semântica por encoder H.264. x264 usa CRF e presets `ultrafast` a `veryslow`. NVENC usa modo de qualidade constante e mapeamento P1–P7. VA-API usa um modo de qualidade suportado pelo driver, com upload de frames quando necessário. Expor os perfis Baseline, Main e High validados. [Manual do FFmpeg](https://ffmpeg.org/ffmpeg.html)
- Detectar `prores_ks` e fazer uma codificação curta de cada perfil ProRes 422 disponível. Usar 4:2:2 de 10 bits e MOV. O muxer do FFmpeg confirma a compatibilidade do áudio copiado em ambos os codecs. Uma falha tipada permite oferecer AAC para H.264 e PCM de 24 bits para ProRes. [Perfis ProRes](https://ffmpeg.org/ffmpeg-codecs.html#ProRes)
- Validar entrada e destino locais. Invocar o executável diretamente, com argumentos estruturados e sem interpolação de shell. Desabilitar stdin e consumir stderr em paralelo com `-progress pipe:1`, com diagnóstico de tamanho limitado. Restringir protocolos aos necessários para arquivos locais e testar mídia malformada e caminhos com espaços.
- Consumir blocos a cada chave `progress`, tolerar campos ausentes e medir unidades com fixtures. Só considerar sucesso com código de saída zero e publicação do arquivo. A fila usa até dois jobs x264 ou ProRes em hosts com múltiplos núcleos, com threads de encoder limitadas por job. Um lote de hardware executa um job por vez no dispositivo escolhido. Cancelar mata e aguarda o subprocesso e remove temporários.
- Gerar temporário com extensão de container no diretório final. Publicar sem sobrescrita com `TempPath::persist_noclobber` e tentar sufixos numéricos em caso de colisão, inclusive entre processos. Preservar entradas e resultados anteriores. Percentual global ponderado por duração quando conhecida, com contagem de itens quando houver duração ausente. ETA e FPS permanecem indisponíveis até medição válida. Redução = `100 × (1 − bytes_saida / bytes_entrada)`, podendo ser negativa quando o arquivo crescer.

## Web e distribuição

A landing do card 4 usa HTML estático, fontes locais e JavaScript compartilhado pelos quatro mockups. Cada demo tem estado próprio e usa arquivos de exemplo. Os controles e os estados reproduzem a interface do card 3. Declarações de desempenho, depoimentos e links de instalação seguem as evidências do card 5 e os insumos do usuário.

O instalador do usuário e o tarball gravam em `~/.local`. RPM, Flatpak e instalação local compartilham o desktop entry, o ícone e os avisos de licença das fontes. O RPM e o tarball usam o FFmpeg e o Qt 6 da distribuição. O Flatpak compila no KDE SDK 6.11 e usa o FFmpeg do runtime, com acesso Wayland/X11, portais, nós DRM e notificações. O script de release gera os três pacotes e o manifesto SHA-256 referenciados na landing. [Distribuição e testes](card-5-distribuicao.md)

No macOS, o mesmo instalador gera `~/Applications/Velox Transcode.app`, incorpora frameworks e plugins Qt e usa o ícone Retina do bundle. O instalador registra os caminhos absolutos de FFmpeg e ffprobe nos recursos do bundle. `cargo run --locked` resolve os executáveis via PATH durante o desenvolvimento. [Instalação no Fedora e macOS](card-9-instalacao.md)

## Instalação no Windows 11 em validação

`install-windows.ps1` compila com Qt 6, usa `windeployqt`, copia FFmpeg e ffprobe para um subdiretório próprio e instala em `%LOCALAPPDATA%` com atalho no menu Iniciar. O motor usa os executáveis instalados ou o PATH no desenvolvimento. A conclusão da fila é mostrada na interface; notificações nativas do Windows e o aceite em sistema Windows continuam pendentes. [Card 10](development-plan.md#card-10--instalação-no-windows-11)
