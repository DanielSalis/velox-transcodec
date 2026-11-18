# Plano de desenvolvimento em cinco cards

**Card 1 implementado e verificado. Cards 2–5 aguardam início.** O protótipo da Fase 1 serve de referência de composição. Cada card tem um agente responsável e entrega código, verificações executadas e pendências com ambiente identificado.

**Dependências.** Card 1 abre os contratos para 2 e 3. Card 3 integra as entregas de 1 e 2. Card 4 pode começar em paralelo com 1 e sincroniza a representação final com 3. Card 5 integra tudo, entrega os artefatos e devolve ao card 4 os nomes, comandos e links verificados para conferência final.

## Card 1 — Fundação Rust e prova de desktop no Fedora

**Entrega.** Criar manifesto Cargo, perfil release, lockfile, esqueleto compilável e tipos de domínio da arquitetura. Fixar versões verificadas de Slint, seu backend desktop e Tokio. Provar abertura de janela com identificação do app, arraste externo de múltiplos arquivos e seletor portal. Registrar requisitos Wayland/XKB e diferenças entre Workstation e Silverblue. O agente valida dependências e alternativas se a compatibilidade do DnD exigir um adaptador.

**Aceite.** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test` passam. Registrar sessão, compositor e versão ao testar entrada múltipla Wayland nativa e X11. Registrar as assinaturas e limitações para os agentes dos cards 2 e 3. [Evidência de aceite e ambientes](card-1-fedora.md).

## Card 2 — Motor FFmpeg e fila assíncrona

**Depende do card 1.** Implementar probe enriquecido, descoberta e teste real de VA-API/NVENC/x264, mapeamentos de qualidade/perfil/preset/áudio/container e execução paralela limitada com cancelamento e erros por item. Entregar progresso por item e global, FPS, ETA e redução final com saídas protegidas contra colisões. Expor comandos e eventos ao card 3.

**Aceite.** Executar transcodes reais usando fixtures sintéticas com ffprobe de saída, incluindo MP4/MKV/MOV, Copy compatível e AAC 192/320, cancelamento e erro. Exercitar metadados ausentes, arquivo inválido, caminhos com espaços, colisão em execução concorrente e falha de encoder. Registrar quais GPUs foram exercitadas, testar a filtragem de hardware indisponível e medir FPS, tempo total e memória em uma amostra reproduzível.

## Card 3 — Interface nativa, controles e notificação

**Depende dos cards 1 e 2 para integração.** Implementar em Slint a janela aprovada, tipografia e tokens, entrada múltipla por drag & drop, seletor portal e arquivos abertos via .desktop. Conectar tabela de metadados, seleção de destino, encoder e opções suportadas, progresso por item e global, cancelamento, conclusão e notificação via D-Bus.

**Aceite.** Verificar seleção de vários arquivos, edição por teclado, estados vazio/erro/execução/conclusão e compatibilidade por encoder. Completar um lote real em Fedora sem bloquear a interface. Verificar notificação e continuidade da UI quando o serviço D-Bus não estiver disponível. Entregar gravação ou capturas e resultados de Wayland e X11.

## Card 4 — Landing page responsiva e mockups

**Pode começar após aprovação visual.** Completar Announcement → Header → Hero → Features alternadas (codecs, presets, fila) → Testimonials (três linhas, autor/cargo e avatar) → Download (Fedora RPM/Build Local, Flatpak, Arch/Tarball) → FAQ (seis respostas abertas) → Footer (marca, três grupos de links, sociais, copyright e nota técnica). Reutilizar o mockup do checkpoint e manter a correspondência com os estados do card 3. Usar depoimentos autorizados e destinos fornecidos; se indisponíveis, registrar a pendência e identificar qualquer conteúdo de revisão como provisório.

**Aceite.** Testar em 1440 e 390 px, sem overflow horizontal. Cada seção de conteúdo após o Hero tem título de duas linhas e uma ação pequena alinhada à última linha, sem CTA no rodapé da seção. Verificar teclado, foco, contraste, funcionamento dos três mockups, seletor H.264, comandos copiáveis e seis respostas visíveis. Revisar disponibilidades, downloads e comandos junto com o card 5.

## Card 5 — Instalação, empacotamento e validação final

**Depende dos cards 1–4 para aceite integrado.** Implementar `install.sh` que verifica FFmpeg/ffprobe e dependências Wayland/XKB do Fedora, faz `cargo build --release --locked`, instala binário, SVG e .desktop em `~/.local` e executa `update-desktop-database`. O .desktop usa `Categories=AudioVideo;Video;`, `MimeType=video/mp4;video/x-matroska;video/webm;video/quicktime;`, `Exec` com múltiplos arquivos e `StartupWMClass` medida. Preparar RPM/build local, Flatpak e tarball para os três canais da página, com release local, instruções e checksums verificáveis.

**Aceite.** Testar script com usuário comum em Fedora Workstation e Silverblue, incluindo reinstalação e dependência ausente. Executar `desktop-file-validate`, abertura por MIME, agrupamento em GNOME/KDE e lote de ponta a ponta. Instalar e testar os pacotes produzidos, com registro de GPUs e rendição da UI por ambiente. Validar comandos exibidos na landing e documentar o estado dos canais de publicação. Publicação externa depende de autorização.
