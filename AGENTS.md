# Agentes de desenvolvimento do Velox Transcode

## Estado e entrada

A Fase 1 entrega a estrutura de trabalho, o brand kit e a prévia do primeiro ecrã. O desenvolvimento dos cinco cards começa após aprovação explícita do usuário. Até lá, trabalhe na prévia e nos documentos que serão avaliados.

Leia [PRODUCT.md](PRODUCT.md), [DESIGN.md](DESIGN.md), [docs/architecture.md](docs/architecture.md) e os critérios do card em [docs/development-plan.md](docs/development-plan.md). Use o card como unidade de entrega.

## Trabalho em paralelo

- O coordenador atribui um card por vez a cada agente, registra dependências e distribui áreas exclusivas de edição. Agentes simultâneos usam branches ou worktrees distintos quando a ferramenta de execução permitir.
- O agente dono do card mantém os contratos sob sua responsabilidade. Registre ajustes nos contratos e alinhe com os consumidores antes do merge. Alterações em arquivos de outro card exigem coordenação com seu dono.
- Cards 1 e 4 podem começar em paralelo após aprovação. Card 2 pode começar assim que as interfaces do card 1 forem integradas. Card 3 integra o motor. Card 5 conclui a integração e registra a evidência de distribuição.
- Mudanças de comportamento, limitações descobertas e resultados de verificação devem acompanhar o handoff. Informe os comandos executados, os ambientes, as lacunas e a próxima dependência acionável.

## Regras do projeto

- A lógica de mídia e de fila fica em Rust, com FFmpeg e ffprobe invocados por vetores de argumentos. O thread da interface apenas consome comandos e eventos.
- Tokens e estados visuais seguem [DESIGN.md](DESIGN.md). O protótipo usa arquivos de demonstração. Resultados de conversão, disponibilidade de hardware, depoimentos e URLs de distribuição precisam de evidência para serem tratados como reais.
- Prefira verificações do card e reutilize a infraestrutura existente. Na Fase 2, rode formatação, Clippy e testes Cargo quando alterar Rust, além das verificações nativas do card. Na web, valide teclado, interações, contraste e os tamanhos de 1440 e 390 px.
- Na conclusão de cada card, revise o diff e entregue resumo, evidência de aceite e limitações. O coordenador atualiza o plano conforme as evidências e resolve dependências entre cards.
