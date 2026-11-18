# Arquitetura proposta

## Fronteiras e propriedade

| Caminho previsto | Responsabilidade | Card |
| --- | --- | --- |
| `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/lib.rs`, `src/domain/` | Pacote binário Rust Edition 2024, contratos e integração de módulos | 1 |
| `src/media/`, `src/queue/`, `tests/media/` | ffprobe, detecção de encoder, parâmetros de FFmpeg, execução e progresso | 2 |
| `src/main.rs`, `src/desktop/`, `ui/` | Janela Slint, entrada de arquivos, UI e ponte de comando/evento | 3 |
| `web/` | Site estático e mockups interativos, com o SVG da marca também usado pela instalação | 4 |
| `install.sh`, `packaging/` | Instalação do usuário, desktop entry, formatos de distribuição e verificação | 5 |

Card 1 cria o manifesto e esqueleto compilável. Cards 2 e 3 coordenam alterações de manifesto e módulos compartilhados. Em fase 2 o perfil release terá `lto = true`, `codegen-units = 1`, `strip = true`, e o pacote utilizará `Cargo.lock`.

## Desktop e ambiente Fedora

**Decisão proposta: Slint com estilo próprio, Winit e renderização por software como perfil inicial.** O layout é majoritariamente estático, o renderer dispensa GPU para a UI e o backend Winit cobre Wayland e X11. A renderização da UI e o encoder de vídeo são independentes. O card 1 mede a versão fixada no host antes de consolidar a escolha. [Backends do Slint](https://docs.slint.dev/latest/docs/rust/slint/docs/cargo_features/) · [Eventos Iced](https://docs.rs/iced/latest/iced/window/enum.Event.html)

O maior risco é receber arraste externo de múltiplos arquivos no Wayland nativo. O Winit tem uma API de DnD upstream. A versão fixada do Slint precisa expô-la e passar uma prova em execução, incluindo X11. Verificar `DropArea` e a leitura de `text/uri-list`, com Nautilus e Dolphin nas sessões testadas. Se a versão estável exigir integração adicional, delimitar um adaptador e registrar a evidência no card 1. Um caminho via XWayland é cobertura de compatibilidade, e a aceitação Wayland nativa continua explícita. Portal resolve o seletor, não o arraste sobre a superfície. [API de drag & drop no Winit](https://github.com/rust-windowing/winit/pull/4571) · [Drag & drop no Slint](https://docs.slint.dev/latest/docs/slint/reference/drag-and-drop/dragarea/)

Usar `rfd` assíncrono com XDG Desktop Portal para seleção múltipla e pasta de destino. Drag & drop, portal e acionamento de arquivos pelo .desktop convergem em `AddFiles`. Usar caminhos locais como `PathBuf` até a camada de apresentação. Identidade provisória `dev.veloxtranscode.Velox` para nome do .desktop, ícone e app ID Wayland, sujeita à confirmação do namespace para publicação. Medir a classe real X11 para `StartupWMClass`. [Slint set_xdg_app_id](https://docs.slint.dev/latest/docs/rust/slint/fn.set_xdg_app_id.html)

## Contratos de integração

Nomes abaixo são propostas para o card 1 congelar nas interfaces Rust. Tipos de domínio são a fronteira entre a UI e o motor.

| Contrato | Campos e comportamento |
| --- | --- |
| `MediaInfo` | ID estável, caminho, stream de vídeo escolhido, codec, largura, altura, taxa racional de FPS, bitrate e duração opcionais, bytes originais e streams de áudio |
| `EncoderCapability` | ID (`Vaapi`, `Nvenc`, `X264`), dispositivo quando aplicável, opções suportadas e motivo de indisponibilidade para diagnóstico |
| `OutputSettings` | Encoder, qualidade 18–28, velocidade, perfil, áudio Copy/AAC 192/AAC 320, container MP4/MKV/MOV e pasta de destino |
| `QueueCommand` | Adicionar caminhos, iniciar seleção com configurações imutáveis por lote e cancelar execução. Validações devolvem erros tipados |
| `QueueEvent` | Probe por item, capacidades, transições e progresso por ID, seguido de resumo final de lote |
| `ProgressSnapshot` | Tempo processado, FPS e velocidade opcionais, percentual/ETA opcionais, bytes finais e redução após conclusão |

O thread da janela aplica os eventos via event loop do Slint. Tokio executa probes, inventário de hardware e transcodes. Canais limitados e coalescência de progresso preservam a responsividade sem perder mudanças de estado ou conclusão. Transições esperadas: `Probing → Ready → Queued → Running → Succeeded | Failed | Cancelled`; erro de metadados também pode encerrar `Probing`. Um `QueueFinished` produz uma notificação D-Bus via `notify-rust`; indisponibilidade do serviço é um evento não fatal.

## Estratégia de mídia

- Invocar ffprobe com saída JSON e selecionar a primeira trilha de vídeo útil, excluindo capas anexas. Tratar `N/A`, taxa indefinida e ausência de duração ou bitrate como valores opcionais. Exibir resumo do stream e duração do arquivo quando disponível. [FFprobe](https://ffmpeg.org/ffprobe.html)
- Cruzar `ffmpeg -encoders` com um encode curto por encoder e por nó VA-API acessível. Enumerar nós `/dev/dri/renderD*`, registrar diagnóstico de inicialização e filtrar opções inviáveis. Ordem inicial determinística VA-API, NVENC, libx264, ajustável na UI. Se libx264 não iniciar, apresentar diagnóstico de build e dependências.
- Os valores 18–28 têm semântica por encoder. x264 usa CRF e presets `ultrafast` a `veryslow`. NVENC usa modo de qualidade constante e mapeamento P1–P7. VA-API usa um modo de qualidade suportado pelo driver, com upload de frames quando necessário. Expor apenas controles validados por encoder e mapear perfis de Compatibilidade, Main e High para opções realmente aceitas. Validar áudio Copy × container por arquivo e oferecer AAC como ação de recuperação. [Manual do FFmpeg](https://ffmpeg.org/ffmpeg.html) · [Opções de codecs](https://ffmpeg.org/ffmpeg-codecs.html)
- Validar entrada e destino locais. Invocar o executável diretamente, com argumentos estruturados e sem interpolação de shell. Desabilitar stdin e consumir stderr em paralelo com `-progress pipe:1`, com diagnóstico de tamanho limitado. Restringir protocolos aos necessários para arquivos locais e testar mídia malformada e caminhos com espaços.
- Consumir blocos a cada chave `progress`, tolerar campos ausentes e medir unidades com fixtures. Só considerar sucesso com código de saída zero e publicação do arquivo. Limitar concorrência inicial a dois jobs globais e no máximo um por dispositivo ou CPU, com suporte para revisão do limite por medição. Cancelar mata e aguarda o subprocesso e remove temporários.
- Gerar temporário com extensão de container no diretório final. Reservar nomes por lote e publicar com semântica atômica sem sobrescrita, incluindo concorrência externa. Preservar entradas e resultados anteriores. Percentual global ponderado por duração quando conhecida, com contagem de itens quando houver duração ausente. ETA e FPS permanecem indisponíveis até medição válida. Redução = `100 × (1 − bytes_saida / bytes_entrada)`, podendo ser negativa quando o arquivo crescer.

## Web e distribuição

O navegador reproduz interações com fixtures identificadas. O card 4 mantém HTML semântico e JavaScript compartilhado entre os mockups, sincronizando nomes, estados e geometria com o card 3. Declarações de desempenho, depoimentos e links de instalação seguem as evidências do card 5 e os insumos do usuário.

A aplicação e seu .desktop são instalados dentro de `~/.local` no host. No Silverblue, compilar em Toolbx quando apropriado e validar dependências de runtime no host. Flatpak precisa de FFmpeg dentro do sandbox, portal, notificação e acesso gráfico/dispositivos compatível com os drivers. RPM e tarball documentam as dependências externas. AV1/HEVC como entradas e H.264 como saída exigem testar o build real de FFmpeg. [Fedora Silverblue](https://docs.fedoraproject.org/en-US/fedora-silverblue/) · [Flatpak, permissões de sandbox](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
