# Brand kit do Velox Transcode

O nome de exibição é **Velox Transcode**. **Velox** é o wordmark curto do cabeçalho e da janela. O [ícone vetorial](web/assets/velox-transcode.svg) usa um V de duas faixas convergentes com acabamento monocromático no azul de ação. Use a marca em Host Grotesk 500 e preserve o desenho no ícone de instalação, no favicon e nos cabeçalhos. O ícone é identificável em 16 px e deve ser exportado com o mesmo viewBox quando necessário.

## Tipografia e estrutura

- **Host Grotesk 500** para display, sem itálico ou peso artificial. Hero e títulos de seção com 62 px a 1440 px, 47 px a 390 px e entrelinha próxima de 1.02. Use Instrument Sans para rótulos, dados de arquivo e ações.
- **Instrument Sans 16 px** para corpo. Metadados compactos entre 11 e 14 px e algarismos tabulares para duração, FPS, tamanho e progresso. As fontes web e licenças OFL estão em [web/assets/fonts](web/assets/fonts). O desktop incorpora as versões TTF das mesmas famílias em [ui/fonts](ui/fonts), cobertas pelas licenças OFL correspondentes. Cabeçalhos nativos usam Host Grotesk 500 entre 16 e 19 px.
- Conteúdo com máximo de 1200 px e mínimo de 24 px de margem lateral. Bandas usam 96 px no topo e 64 px na base a 1440 px, 64 px e 48 px a 390 px. O fundo da página é contínuo.
- Controles e superfícies têm cantos de 8 px e divisões planas de 1 px. Use azul para ação primária, foco e progresso. Ícones de interface têm traços de 1,5 px. O mockup usa cabeçalho de janela, lista de vídeos, painel de saída e faixa de processamento que o desktop deve reproduzir.
- Após o Hero, cada seção de conteúdo abre com um título de duas linhas à esquerda e um controle pequeno no extremo direito, alinhado à última linha. Em 390 px, o controle pode ser representado por um ícone com nome acessível e área de toque. O rodapé concentra os links estruturais da marca.

## Tokens e contraste

| Token | Valor | Uso |
| --- | --- | --- |
| Ground | `#0f1012` | Fundo contínuo da página e da janela |
| Surface | `#161719` | Cabeçalho, janela e controles |
| Hairline | `#252729` | Separadores e bordas de superfície |
| Primary | `#faf9fa` | Títulos e rótulos primários |
| Secondary | `#e5e5e4` | Corpo, valores e controles secundários |
| Muted | `#d7d7d6` | Metadados e mensagens auxiliares |
| Accent | `#b4d3ff` | Ação primária, links, foco, seleção e progresso |
| Inverse text | `#040404` | Rótulo sobre Accent ou Inverse ground |
| Inverse ground | `#efedef` | Superfície clara institucional |

Valores aproximados por par de texto e fundo, arredondados a partir de sRGB. O sinal em APCA indica polaridade. A meta é magnitude `|Lc| ≥ 75` e WCAG AA simultaneamente.

| Texto | Sobre Ground | Sobre Surface |
| --- | --- | --- |
| Primary | 18,13:1 / Lc −103,7 | 17,08:1 / Lc −103,1 |
| Secondary | 15,10:1 / Lc −90,5 | 14,23:1 / Lc −90,0 |
| Muted | 13,22:1 / Lc −81,8 | 12,45:1 / Lc −81,3 |
| Accent | 12,42:1 / Lc −78,1 | 11,70:1 / Lc −77,5 |
| Inverse text | Sobre Accent: 13,38:1 / Lc +79,1 | Sobre Inverse ground: 17,60:1 / Lc +95,7 |

Use Hairline para divisão decorativa. Contornos que identificam áreas editáveis usam `#666b72` sobre os fundos escuros para contraste não textual, e foco visível usa Accent. Estados selecionados usam cor e um indicador textual ou visual de forma. O texto mantém os pares acima nos estados de hover e foco. [WCAG 2.2, contraste textual](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) · [WCAG 2.2, contraste não textual](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html) · [APCA-W3](https://github.com/Myndex/apca-w3/blob/master/README.md)

## Primeiro ecrã

Announcement em linha única para a prévia Fedora, cabeçalho recuado com a marca, quatro links e ação primária. Hero centralizado com título em duas linhas, duas ações, legenda de privacidade e três capacidades. A janela abaixo resume a experiência de uso com arquivos de exemplo, metadados por linha, controles de saída H.264 e fila executável. Em 390 px, os quatro links ficam em um menu e a janela empilha lista, configuração e fila. A seleção, os valores do formulário e o início da fila são acionáveis por teclado.

A ação de instalação abre a informação de disponibilidade no checkpoint. A versão completa usa os canais e comandos verificados no card 5.
