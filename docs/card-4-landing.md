# Card 4 — Landing page responsiva

Abra [web/index.html](../web/index.html) em um navegador. A página também pode ser servida por HTTP com `python3 -m http.server 8000 --bind 127.0.0.1 --directory web`. O HTML, o JavaScript e as fontes são servidos pelo próprio diretório.

A página apresenta a prévia 0.1.0, o Hero aprovado e as seções Recursos, Depoimentos, Download e FAQ, seguidas do rodapé. O Hero mantém a demonstração completa. Em Recursos, as três janelas isolam a leitura de codecs, os presets H.264 e a execução da fila. A mesma lógica alimenta os mockups e cada janela guarda sua própria seleção e progresso. Todo processamento da página usa dados de exemplo.

## Verificação

| Verificação | Resultado |
| --- | --- |
| Desktop em 1440 px e mobile em 390 px | Sem rolagem horizontal. Títulos usam Host Grotesk 500 em 62 e 47 px. Cada seção de conteúdo tem duas linhas de título e uma ação à direita alinhada à última linha. |
| Recursos, Depoimentos, Download e FAQ | Três linhas alternadas no desktop, três citações com autoria e avatar tipográfico, três comandos copiáveis e seis respostas abertas em duas colunas. |
| Interações no Chromium | Navegação por teclado, menu mobile, seleção e inclusão de vídeos, atualização de presets e qualidade, início, conclusão e cancelamento de fila. Cópia dos três comandos, caminho de compatibilidade e seleção manual conferidos. |
| Abertura direta e por HTTP | Modelos, ícone, fontes e links locais carregaram sem erro de JavaScript. Larguras de 320 e 900 px também foram verificadas. |

Os nove pares de cores de texto medidos na renderização inicial atingiram pelo menos 11,70:1 e APCA |Lc| 77,5. A inspeção visual cobriu 1440 e 390 px. [Matriz do brand kit](../DESIGN.md)

## Entrega para a distribuição

Os depoimentos têm citações e avatares tipográficos identificados como exemplos com personagens fictícios. Substitua esse conteúdo quando houver material autorizado. Os links de comunidades no rodapé são do Fedora e do FFmpeg.

O card 5 atualizou o Download com os artefatos da versão 0.1.0, os comandos de instalação e os checksums. Gere os arquivos em `web/downloads/` para testar os links em um novo checkout. [Distribuição e checksums](card-5-distribuicao.md)
