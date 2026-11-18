# Plano de desenvolvimento do Velox Transcode

**Cards 1 a 5 implementados.** O card 5 entrega os pacotes locais. A matriz registra as verificações pendentes em Silverblue e nas docks. O protótipo da Fase 1 serve de referência de composição. [Validação da distribuição](card-5-distribuicao.md)

**Cards 6 e 7 propostos.** Ambos aguardam aprovação para implementação.

**Dependências.** Card 1 abre os contratos para 2 e 3. Card 3 integra as entregas de 1 e 2. Card 4 pode começar em paralelo com 1 e sincroniza a representação final com 3. Card 5 integra tudo, entrega os artefatos e devolve ao card 4 os nomes, comandos e links verificados para conferência final.

## Card 1 — Fundação Rust e prova de desktop no Fedora

**Entrega.** Criar manifesto Cargo, perfil release, lockfile, esqueleto compilável e tipos de domínio da arquitetura. Fixar versões verificadas de Slint, seu backend desktop e Tokio. Provar abertura de janela com identificação do app, arraste externo de múltiplos arquivos e seletor portal. Registrar requisitos Wayland/XKB e diferenças entre Workstation e Silverblue. O agente valida dependências e alternativas se a compatibilidade do DnD exigir um adaptador.

**Aceite.** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test` passam. Registrar sessão, compositor e versão ao testar entrada múltipla Wayland nativa e X11. Registrar as assinaturas e limitações para os agentes dos cards 2 e 3. [Evidência de aceite e ambientes](card-1-fedora.md).

## Card 2 — Motor FFmpeg e fila assíncrona

**Depende do card 1.** Implementar probe enriquecido, descoberta e teste real de VA-API/NVENC/x264, mapeamentos de qualidade/perfil/preset/áudio/container e execução paralela limitada com cancelamento e erros por item. Entregar progresso por item e global, FPS, ETA e redução final com saídas protegidas contra colisões. Expor comandos e eventos ao card 3.

**Aceite.** Executar transcodes reais usando fixtures sintéticas com ffprobe de saída, incluindo MP4/MKV/MOV, Copy compatível e AAC 192/320, cancelamento e erro. Exercitar metadados ausentes, arquivo inválido, caminhos com espaços, colisão em execução concorrente e falha de encoder. Registrar quais GPUs foram exercitadas, testar a filtragem de hardware indisponível e medir FPS, tempo total e memória em uma amostra reproduzível. [Evidências e integração](card-2-media.md).

## Card 3 — Interface nativa, controles e notificação

**Depende dos cards 1 e 2 para integração.** Implementar em Slint a janela aprovada, tipografia e tokens, entrada múltipla por drag & drop, seletor portal e arquivos abertos via .desktop. Conectar tabela de metadados, seleção de destino, encoder e opções suportadas, progresso por item e global, cancelamento, conclusão e notificação via D-Bus.

**Aceite.** Verificar seleção de vários arquivos, edição por teclado, estados vazio/erro/execução/conclusão e compatibilidade por encoder. Completar um lote real em Fedora sem bloquear a interface. Verificar notificação e continuidade da UI quando o serviço D-Bus não estiver disponível. Entregar gravação ou capturas e resultados de Wayland e X11. [Evidências do desktop](card-3-desktop.md).

## Card 4 — Landing page responsiva e mockups

**Pode começar após aprovação visual.** Completar Announcement → Header → Hero → Features alternadas (codecs, presets, fila) → Testimonials (três linhas, autor/cargo e avatar) → Download (Fedora RPM/Build Local, Flatpak, Arch/Tarball) → FAQ (seis respostas abertas) → Footer (marca, três grupos de links, sociais, copyright e nota técnica). Reutilizar o mockup do checkpoint e manter a correspondência com os estados do card 3. Usar depoimentos autorizados e destinos fornecidos; se indisponíveis, registrar a pendência e identificar qualquer conteúdo de revisão como provisório.

**Aceite.** Testar em 1440 e 390 px, sem overflow horizontal. Cada seção de conteúdo após o Hero tem título de duas linhas e uma ação pequena alinhada à última linha, sem CTA no rodapé da seção. Verificar teclado, foco, contraste, funcionamento dos três mockups, seletor H.264, comandos copiáveis e seis respostas visíveis. Revisar disponibilidades, downloads e comandos junto com o card 5. [Evidências da landing](card-4-landing.md).

## Card 5 — Instalação, empacotamento e validação final

**Depende dos cards 1–4 para aceite integrado.** Implementar `install.sh` que verifica FFmpeg/ffprobe e dependências Wayland/XKB do Fedora, faz `cargo build --release --locked`, instala binário, SVG e .desktop em `~/.local` e executa `update-desktop-database`. O .desktop usa `Categories=AudioVideo;Video;`, `MimeType=video/mp4;video/x-matroska;video/webm;video/quicktime;`, `Exec` com múltiplos arquivos e `StartupWMClass` medida. Preparar RPM/build local, Flatpak e tarball para os três canais da página, com release local, instruções e checksums verificáveis.

**Aceite.** Testar script com usuário comum em Fedora Workstation e Silverblue, incluindo reinstalação e dependência ausente. Executar `desktop-file-validate`, abertura por MIME, agrupamento em GNOME/KDE e lote de ponta a ponta. Instalar e testar os pacotes produzidos, com registro de GPUs e rendição da UI por ambiente. Validar comandos exibidos na landing e documentar o estado dos canais de publicação. Publicação externa depende de autorização.

**Entrega local.** O instalador, o RPM, o Flatpak e o tarball foram executados nos ambientes registrados. A validação formal em Silverblue e o agrupamento visual nas docks GNOME/KDE permanecem pendentes. [Comandos, resultados e limites da validação](card-5-distribuicao.md)

## Card 6 — Barra de título própria

**Proposta visual.** Transformar o cabeçalho atual do Velox na única barra de título da janela. Manter o ícone e o nome à esquerda sobre a superfície escura do brand kit, com três botões de ícone à direita, na ordem **Minimizar**, **Maximizar ou restaurar** e **Fechar**. Usar traços de 1,5 px, foco azul, estados visíveis ao passar o cursor e pressionar, e nomes acessíveis e dicas em português. [Brand kit](../DESIGN.md)

**Entrega.** Usar `no-frame` e `WindowMoveArea` do Slint para desenhar e mover a janela. Usar ações nativas de minimizar, maximizar e restaurar, com ícone e rótulo sincronizados também quando o compositor mudar o estado. Duplo clique na área livre alterna maximizar e restaurar. Encaminhar o botão Fechar ao pedido de fechamento existente, que cancela e aguarda a fila ativa. Provar o redimensionamento pelas bordas no backend Qt no início do card. A propriedade `resize-border-width` do Slint 1.19 depende do Winit, então, se a janela sem moldura exigir integração adicional, validar uma ponte pontual para `QWindow::startSystemResize`. [Janela Slint](https://docs.slint.dev/latest/docs/slint/reference/window/window/) · [Geometria interativa no Qt](https://doc.qt.io/qt-6.8/qwindow.html#startSystemResize)

**Aceite.** No Fedora em Wayland e X11, confirmar que aparece uma única barra, que o mouse move e redimensiona a janela e que os três botões funcionam por mouse e teclado. Minimizar mantém um lote em execução. Fechar aguarda o cancelamento e a limpeza dos temporários. Verificar também restauração, atalhos do compositor, tamanho mínimo, escala de 100% e 150%, identificação no launcher e arraste externo de arquivos. Registrar capturas da janela restaurada e maximizada.

## Card 7 — Codec de saída Apple ProRes

**Proposta de produto.** Adicionar **Codec de saída** com H.264 como opção inicial e ProRes como alternativa. Ao escolher ProRes, exibir **MOV**, encoder **ProRes · CPU**, perfis **422 Proxy, 422 LT, 422 e 422 HQ**, com **422** selecionado inicialmente, e áudio **PCM 24 bits** ou **Copiar** quando compatível. Os perfis substituem os controles de CRF, qualidade constante e preset nessa opção. Identificar ProRes como formato para edição e informar que o tamanho pode aumentar. [Perfis ProRes no FFmpeg](https://ffmpeg.org/ffmpeg-codecs.html#ProRes)

**Entrega.** Tipar a configuração de vídeo em Rust por codec para validar as combinações. Detectar `prores_ks` pelo FFmpeg e confirmar os perfis disponíveis com uma codificação curta. Exibir o motivo quando o encoder estiver indisponível. Gerar MOV com vídeo 4:2:2 de 10 bits, manter o fluxo de fila e usar o sufixo `-prores.mov` com a proteção existente contra sobrescrita. Para áudio Copy incompatível, oferecer PCM como recuperação. Ao voltar a H.264, restaurar os ajustes H.264 escolhidos na sessão.

**Aceite.** Converter pelo aplicativo uma amostra para cada perfil oferecido e conferir com ffprobe o codec, o perfil, o formato de pixel, o contêiner e o áudio. Incluir arquivo sem áudio, áudio copiado compatível, encoder ausente, colisão de nomes, cancelamento e regressão H.264. Verificar o caminho com o FFmpeg do Fedora e no Flatpak. Atualizar o manual de uso e a landing após confirmar a disponibilidade em cada canal.

Os cards podem ser desenvolvidos separadamente. O card 6 trabalha no cabeçalho e nos eventos da janela. O card 7 trabalha no painel de saída, no domínio e no motor de mídia. Integrar as mudanças no arquivo Slint compartilhado e executar os testes Rust e do desktop após a integração.
