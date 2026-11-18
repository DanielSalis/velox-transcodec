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

## Desktop e ambiente Fedora

**Slint 1.19.0 com estilo próprio e backend Qt 6.** O backend Winit do Slint 1.19.0 usa a série 0.30 e não expõe arraste de arquivos externo em Wayland nativo. O backend Qt encaminha eventos nativos para `DropArea` e popula `DataTransfer::file_paths()` a partir dos arquivos oferecidos. A Qt é uma dependência nativa em runtime. O backend de janela permanece independente da aceleração do encoder de vídeo. [Backend Qt do Slint 1.19.0](https://docs.rs/crate/i-slint-backend-qt/1.19.0/source/qt_window.rs) · [Arraste em Wayland no Winit](https://github.com/rust-windowing/winit/pull/4571)

Os seletores de vídeos e de pasta de destino usam `rfd` assíncrono via XDG Desktop Portal. Arraste, portal e argumentos de abertura convergem em `QueueCommand::AddFiles`. Os valores permanecem `PathBuf` até a apresentação. Os seletores abrem sem uma janela pai exportada pelo backend Qt. [rfd 0.18.0](https://docs.rs/rfd/0.18.0/rfd/)

Identidade provisória `dev.veloxtranscode.Velox` para o nome do .desktop, ícone e app ID Wayland, sujeita à confirmação do namespace para publicação. A classe medida em X11 nesta compilação foi `velox-transcode`; o card 5 repete a medição no artefato instalado antes de definir `StartupWMClass`. [Ambientes e evidências](card-1-fedora.md)

## Contratos de integração

As interfaces estão em [`src/domain`](../src/domain). Elas são independentes de Slint e FFmpeg.

| Contrato | Campos e comportamento |
| --- | --- |
| `MediaInfo` | `MediaId` da sessão, caminho, stream de vídeo e streams de áudio, duração/bitrate opcionais e bytes originais. FPS racional com numerador e denominador positivos quando disponível |
| `EncoderCapability` | `EncoderTarget` com nó VA-API, índice NVENC ou x264. `support` descreve os controles validados ou o motivo tipado de falha |
| `OutputSettings` | Encoder, `Quality` validada em 18–28, preset tipado, perfil, áudio, container e pasta de destino |
| `QueueCommand` | Adicionar arquivos, sondar encoders, iniciar uma seleção com configurações do lote e cancelar por `BatchId` |
| `QueueEvent` | Entrada, probe, capacidades, estado, progresso e conclusão. Erros tipados distinguem rejeição de comando e falha de trabalho |
| `ProgressSnapshot`, `BatchProgress` e `JobOutput` | Tempo processado, FPS, velocidade, ETA e percentual. Percentuais usam a escala 0–100. A saída inclui destino, bytes e redução a partir do tamanho final |

A interface recebe `QueueEvent` de [`queue::spawn`](../src/queue/mod.rs) e aplica os eventos num único consumidor no event loop do Slint. O modelo da tabela atualiza por ID de mídia e a UI desabilita mudanças no lote ativo. Tokio executa probes, inventário de hardware e transcodes. Canais limitados e coalescência de progresso preservam a responsividade. `ItemAdded` inicia a sondagem e `MediaProbed` sinaliza que o item está pronto. Falha de probe, fila, execução e estados terminais chegam em `ItemChanged`. `BatchFinished` fornece o resumo para uma notificação D-Bus via `notify-rust`; indisponibilidade do serviço é um evento não fatal.

## Estratégia de mídia

- Invocar ffprobe com saída JSON e selecionar a primeira trilha de vídeo útil, excluindo capas anexas. Tratar `N/A`, taxa indefinida e ausência de duração ou bitrate como valores opcionais. Exibir resumo do stream e duração do arquivo quando disponível. [FFprobe](https://ffmpeg.org/ffprobe.html)
- Cruzar `ffmpeg -encoders` com um encode curto por encoder e por nó VA-API acessível. Enumerar nós `/dev/dri/renderD*`, registrar diagnóstico de inicialização e filtrar opções inviáveis. Ordem inicial determinística VA-API, NVENC, libx264, ajustável na UI. Se libx264 não iniciar, apresentar diagnóstico de build e dependências.
- Os valores 18–28 têm semântica por encoder. x264 usa CRF e presets `ultrafast` a `veryslow`. NVENC usa modo de qualidade constante e mapeamento P1–P7. VA-API usa um modo de qualidade suportado pelo driver, com upload de frames quando necessário. Expor apenas controles validados por encoder e mapear perfis de Compatibilidade, Main e High para opções realmente aceitas. A compatibilidade Copy × container é confirmada pelo muxer do FFmpeg por arquivo. Em caso de incompatibilidade identificada, `JobFailureKind::AudioCopyIncompatible` permite oferecer AAC como recuperação. [Manual do FFmpeg](https://ffmpeg.org/ffmpeg.html) · [Opções de codecs](https://ffmpeg.org/ffmpeg-codecs.html)
- Validar entrada e destino locais. Invocar o executável diretamente, com argumentos estruturados e sem interpolação de shell. Desabilitar stdin e consumir stderr em paralelo com `-progress pipe:1`, com diagnóstico de tamanho limitado. Restringir protocolos aos necessários para arquivos locais e testar mídia malformada e caminhos com espaços.
- Consumir blocos a cada chave `progress`, tolerar campos ausentes e medir unidades com fixtures. Só considerar sucesso com código de saída zero e publicação do arquivo. A fila usa até dois jobs x264 em hosts com múltiplos núcleos, com threads de encoder limitadas por job. Um lote de hardware executa um job por vez no dispositivo escolhido. Cancelar mata e aguarda o subprocesso e remove temporários.
- Gerar temporário com extensão de container no diretório final. Publicar sem sobrescrita com `TempPath::persist_noclobber` e tentar sufixos numéricos em caso de colisão, inclusive entre processos. Preservar entradas e resultados anteriores. Percentual global ponderado por duração quando conhecida, com contagem de itens quando houver duração ausente. ETA e FPS permanecem indisponíveis até medição válida. Redução = `100 × (1 − bytes_saida / bytes_entrada)`, podendo ser negativa quando o arquivo crescer.

## Web e distribuição

A landing do card 4 usa HTML estático, fontes locais e JavaScript compartilhado pelos quatro mockups. Cada demo tem estado próprio e usa arquivos de exemplo. Os controles e os estados reproduzem a interface do card 3. Declarações de desempenho, depoimentos e links de instalação seguem as evidências do card 5 e os insumos do usuário.

A aplicação e seu .desktop são instalados dentro de `~/.local` no host. No Silverblue, compilar em Toolbx quando apropriado e validar dependências de runtime no host. Flatpak precisa de FFmpeg dentro do sandbox, portal, notificação e acesso gráfico/dispositivos compatível com os drivers. RPM e tarball documentam as dependências externas. AV1/HEVC como entradas e H.264 como saída exigem testar o build real de FFmpeg. [Fedora Silverblue](https://docs.fedoraproject.org/en-US/fedora-silverblue/) · [Flatpak, permissões de sandbox](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
