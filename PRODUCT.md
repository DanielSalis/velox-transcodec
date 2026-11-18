# Produto

<!-- impeccable:product-schema 1 -->

## Plataforma

web

A página pública apresenta o Velox Transcode. A aplicação é nativa para Linux, com Fedora Workstation e Silverblue como ambientes-alvo.

## Stack

Rust Edition 2024, Tokio, Slint e subprocessos FFmpeg/ffprobe. A página começa com HTML, CSS e JavaScript estáticos, escolha proposta para manter o mockup executável localmente. A decisão de backend e sua prova de compatibilidade estão na [arquitetura](docs/architecture.md).

## Público e objetivo

Pessoas que convertem arquivos de vídeo no Fedora precisam inspecionar a origem, ajustar o H.264 de saída e executar lotes com progresso legível. O Velox reúne esses passos em uma janela nativa e usa os encoders que conseguirem operar no ambiente.

## Fluxo

1. Adicionar vários arquivos por arraste ou seletor nativo e ler codec, resolução, FPS, bitrate e duração.
2. Ajustar encoder, qualidade 18–28, velocidade, perfil H.264, áudio e container.
3. Executar o lote, acompanhar progresso, FPS, ETA e redução após conclusão e receber uma notificação de sistema.

O trabalho fica local. A página apresenta a experiência com componentes HTML/CSS/JS interativos e arquivos de demonstração.

## Escopo e evidência

A Fase 1 é um checkpoint para aprovação do brand kit, de Announcement + Header + Hero com mockup e do plano de cinco cards. A Fase 2 entrega app, instalação local no Fedora e landing page completa. A distribuição deve refletir os artefatos realmente produzidos. A demonstração usa metadados, disponibilidade e resultados fictícios identificados. Depoimentos, avatares de autores, benchmarks e canais oficiais dependem de insumos verificáveis.

O nome de exibição proposto é **Velox Transcode** e o nome do binário é `velox-transcode`. A primeira versão da página está em português do Brasil. O briefing fixa os tokens, as duas fontes, o contraste e a estrutura descrita em [DESIGN.md](DESIGN.md).
