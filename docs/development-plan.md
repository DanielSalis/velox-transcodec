# Plano de desenvolvimento do Velox Transcode

**Cards 1 a 5 implementados.** O card 5 entrega os pacotes locais. A matriz registra as verificações pendentes em Silverblue e nas docks. O protótipo da Fase 1 serve de referência de composição. [Validação da distribuição](card-5-distribuicao.md)

**Cards 6 a 8 implementados.** A barra própria foi validada em Wayland e X11, a saída ProRes foi verificada no Fedora e no Flatpak e o painel H.264 ganhou rótulos e ajuda de contexto. [Card 6](card-6-barra-de-titulo.md) · [Card 7](card-7-prores.md) · [Card 8](card-8-ajustes.md)

**Card 9 implementado.** O instalador gera o atalho no Fedora e o bundle de usuário no macOS. A matriz registra o aceite no Fedora Workstation e as verificações nativas pendentes no macOS e no Silverblue. [Card 9](card-9-instalacao.md)

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

**Entrega.** Barra Slint com estados sincronizados com o compositor e redimensionamento nativo pelas quatro bordas e pelos quatro cantos. O fechamento durante um lote foi testado com FFmpeg real. [Verificações e capturas](card-6-barra-de-titulo.md)

## Card 7 — Codec de saída Apple ProRes

**Proposta de produto.** Adicionar **Codec de saída** com H.264 como opção inicial e ProRes como alternativa. Ao escolher ProRes, exibir **MOV**, encoder **ProRes · CPU**, perfis **422 Proxy, 422 LT, 422 e 422 HQ**, com **422** selecionado inicialmente, e áudio **PCM 24 bits** ou **Copiar** quando compatível. Os perfis substituem os controles de CRF, qualidade constante e preset nessa opção. Identificar ProRes como formato para edição e informar que o tamanho pode aumentar. [Perfis ProRes no FFmpeg](https://ffmpeg.org/ffmpeg-codecs.html#ProRes)

**Entrega.** Tipar a configuração de vídeo em Rust por codec para validar as combinações. Detectar `prores_ks` pelo FFmpeg e confirmar os perfis disponíveis com uma codificação curta. Exibir o motivo quando o encoder estiver indisponível. Gerar MOV com vídeo 4:2:2 de 10 bits, manter o fluxo de fila e usar o sufixo `-prores.mov` com a proteção existente contra sobrescrita. Para áudio Copy incompatível, oferecer PCM como recuperação. Ao voltar a H.264, restaurar os ajustes H.264 escolhidos na sessão.

**Aceite.** Converter pelo aplicativo uma amostra para cada perfil oferecido e conferir com ffprobe o codec, o perfil, o formato de pixel, o contêiner e o áudio. Incluir arquivo sem áudio, áudio copiado compatível, encoder ausente, colisão de nomes, cancelamento e regressão H.264. Verificar o caminho com o FFmpeg do Fedora e no Flatpak. Atualizar o manual de uso e a landing após confirmar a disponibilidade em cada canal.

**Entrega local.** Os quatro perfis produziram MOV com ProRes 4:2:2 de 10 bits na interface nativa e no Flatpak. Os três artefatos de download e seus checksums foram atualizados. [Resultados e instruções](card-7-prores.md)

## Card 8 — Ajustes de vídeo explicados no painel de saída

**Objetivo.** Mostrar o que muda ao ajustar qualidade, preset e perfil H.264, com rótulos curtos no lugar da decisão e explicações adicionais sob demanda. Integrar este card depois do card 7, que também altera o painel de saída.

**Proposta visual.** Manter a barra de qualidade em largura total e colocar os dois seletores abaixo, lado a lado. Usar o azul do brand kit para o valor selecionado e o foco. Tratar os rótulos das pontas e as descrições dos seletores como texto secundário para preservar a hierarquia do painel. [Brand kit](../DESIGN.md)

| Controle | Texto visível | Explicação sob demanda |
| --- | --- | --- |
| **Qualidade do vídeo** | Valor técnico junto ao título, como **CQ 22** no NVENC, **CRF 22** no x264 ou **QP 22** no VA-API. Pontas da barra **Mais detalhes (18)** e **Arquivo menor (28)**. | Valores menores preservam mais detalhes e tendem a gerar arquivos maiores. O tamanho final depende do vídeo. CQ, CRF e QP são controles próprios de cada encoder. |
| **Velocidade (preset)** | Descrever a opção ao lado do nome técnico no menu, como **P1 · Mais rápido**, **P4 · Equilibrado** e **P7 · Mais lento**. No x264, aplicar a mesma convenção a `ultrafast`, `medium` e `veryslow`. No VA-API, exibir **Automático** e informar que o encoder gerencia o ajuste. | Presets rápidos reduzem o tempo do lote. Presets lentos dedicam mais processamento à compressão. |
| **Perfil H.264** | Manter **Baseline**, **Main** e **High** com uma indicação curta por opção. Usar **Aparelhos antigos**, **Compatibilidade intermediária** e **Aparelhos modernos** como descrições iniciais. | O perfil determina as técnicas de compressão permitidas e a compatibilidade com o aparelho de destino. |

**Entrega.** Reaproveitar os controles Slint e acrescentar uma ação compacta **Entenda os ajustes** que abre as três explicações. Vincular os rótulos aos valores do motor, com qualidade de 18 a 28 e valor inicial 22. Atualizar os textos ao trocar de encoder. Aplicar a ajuda de perfil H.264 ao modo H.264 e usar os rótulos de perfil do card 7 no modo ProRes.

**Aceite.** Conferir as três variantes de encoder, incluindo VA-API em Automático, e a troca entre H.264 e ProRes. Verificar contraste, legibilidade e ausência de truncamento na janela mínima de 880×640 px lógicos a 100% e 150%. Testar a barra, os seletores e a ajuda com mouse e teclado, incluindo foco visível, fechar com Esc e devolver o foco à ação de ajuda. Confirmar que os valores enviados à fila correspondem aos selecionados e entregar uma captura para revisão visual.

**Entrega.** A barra mostra os efeitos de 18 e 28. Os seletores exibem o efeito de velocidade e a compatibilidade do perfil. A ajuda abre dentro da janela e pode ser reaberta pelo teclado depois de Esc. [Verificação e capturas](card-8-ajustes.md)

## Card 9 — Instalação no Fedora e macOS com integração ao desktop

**Objetivo.** Instalar o Velox na conta do usuário a partir de `./install.sh`, abrir pelo menu de aplicativos ou Finder e identificar facilmente a janela em execução na barra de tarefas ou Dock.

**Instalador.** Detectar o sistema, verificar dependências e reaproveitar a instalação release com `cargo build --release --locked` ou com o binário do pacote. Preparar os arquivos antes de substituir a instalação existente. Testar reinstalação e apresentar mensagens que indiquem como resolver dependências ausentes.

**Fedora.** Reaproveitar o instalador de usuário para `~/.local` e documentar o caminho Flatpak no Silverblue. Instalar o desktop entry `dev.veloxtranscode.Velox.desktop` com `Exec` para vários arquivos, tipos MIME e `Icon=dev.veloxtranscode.Velox`. Instalar o SVG e os PNG no tema `hicolor` e atualizar o cache de tipos de arquivo. Associar a janela pelo app ID `dev.veloxtranscode.Velox` em Wayland e pela classe medida `velox-transcode` em X11. [Modelo do desktop entry](../packaging/dev.veloxtranscode.Velox.desktop) · [Especificação freedesktop](https://specifications.freedesktop.org/desktop-entry/latest-single/) · [Tema de ícones](https://specifications.freedesktop.org/icon-theme/)

**macOS.** Provar a compilação e adaptar a integração com o sistema, incluindo seletor de arquivos, abertura pelo Finder, controles de janela e notificação ao concluir. Instalar `~/Applications/Velox Transcode.app` com o executável nativo, as bibliotecas e plugins Qt necessários e `Contents/Info.plist` com nome, versão, `CFBundleIdentifier=dev.veloxtranscode.Velox`, tipos de documento e `CFBundleIconFile=Velox.icns`. Abrir como aplicativo gráfico regular para exibir o ícone no Dock. Garantir que a janela aberta pelo Finder encontre FFmpeg e ffprobe sem depender do PATH do terminal. Validar primeiro em Apple Silicon e registrar as arquiteturas verificadas. [Estrutura do bundle](https://developer.apple.com/documentation/bundleresources/placing-content-in-a-bundle) · [Implantação do Qt](https://doc.qt.io/qt-6/macos-deployment.html)

**Ícones prontos.** O SVG da marca é a origem das versões PNG Fedora em 16, 24, 32, 48, 64, 128 e 256 px. O conjunto macOS inclui `Velox.icns` e o `.iconset` com versões normais e Retina de 16 a 512 pt. A versão macOS tem margem transparente para o símbolo respirar no Dock. [PNG para Fedora](../packaging/icons/fedora/hicolor/) · [Ícone macOS](../packaging/icons/macos/Velox.icns) · [Tamanhos de ícone macOS](https://developer.apple.com/library/archive/documentation/General/Conceptual/MOSXAppProgrammingGuide/CommonAppBehaviors/CommonAppBehaviors.html)

**Aceite.** Instalar como usuário comum e repetir a instalação em Fedora e macOS. Validar os registros com `desktop-file-validate` e `plutil`, confirmar a resolução dos ícones e abrir o app pelo GNOME/KDE e pelo Finder. Abrir um vídeo pelo menu **Abrir com** e concluir um lote a partir desse ponto. Confirmar um único ícone por aplicativo ao abrir, minimizar e restaurar a janela. Verificar a legibilidade do ícone em tamanho pequeno, em fundos claro e escuro, e conferir que é possível fixá-lo e reabrir o Velox pelo Dock ou pela barra de tarefas.

**Evidências.** [Instalação, integração com o desktop e matriz de verificação](card-9-instalacao.md)

## Card 10 — Instalação no Windows 11

**Entrega em validação.** `install-windows.ps1` prepara dependências ausentes com winget e aqtinstall no Windows x64, compila o aplicativo, reúne Qt e FFmpeg e instala por usuário com atalho no menu Iniciar. A preparação automática ainda precisa de teste nativo. O aplicativo procura FFmpeg e ffprobe na instalação e recorre ao PATH no desenvolvimento. As notificações nativas do Windows ainda estão pendentes.

**Aceite pendente em Windows 11.** Compilar com Qt 6 e MSVC, instalar e reinstalar como usuário comum, abrir pelo menu Iniciar, testar seleção e arraste de múltiplos arquivos, converter H.264 e ProRes, verificar cancelamento e remover a instalação. Em Windows build 26300, AMD64, a execução do instalador parou na verificação: Cargo, qmake, FFmpeg e cl.exe não estavam no PATH. O repositório estava em um caminho de rede WSL; será necessária uma cópia local para compilar. Nenhum pacote foi produzido nessa tentativa. Registrar versões de Qt, compilador e FFmpeg quando disponíveis.
