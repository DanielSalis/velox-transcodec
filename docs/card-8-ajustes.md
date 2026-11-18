# Card 8 — Ajustes de vídeo no painel de saída

Na saída H.264, **Mais detalhes (18)** e **Arquivo menor (28)** identificam as pontas da barra. O valor atual aparece junto a **Qualidade do vídeo**, com a sigla do encoder selecionado. Valores menores tendem a preservar mais detalhes e a gerar arquivos maiores. O tamanho final depende do vídeo.

Os menus de **Velocidade (preset)** e **Perfil H.264** mostram o nome técnico junto de uma descrição. Com o menu fechado, a descrição selecionada aparece abaixo do campo. No VA-API, **Automático** indica que o encoder gerencia o ajuste. **Entenda os ajustes** reúne as três explicações. Feche a ajuda com Esc ou pelo botão **Fechar** para retornar ao controle de abertura.

## Verificação

| Ambiente | Resultado |
| --- | --- |
| Fedora 44, Qt X11, 880×640 a 100% | Ajustes por mouse e teclado, limites da barra e foco da ajuda verificados. Uma conversão real acionada pela interface passou ao FFmpeg NVENC com CQ 23, P5 e Main. O ffprobe confirmou H.264 Main. |
| Fedora 44, Qt X11, 880×640 lógicos a 150% | O menu x264 apresentou os nove presets e a ajuda coube na janela. Uma conversão real usou CRF 21, veryslow e Baseline. O ffprobe confirmou H.264 Constrained Baseline. |
| Fedora 44, Weston 15 e Qt Wayland | A ajuda abriu com o mouse. Esc fechou o painel e Espaço o reabriu a partir do foco restaurado. |
| Flatpak de teste, KDE Platform 6.11 | O VA-API exibiu QP 22 e preset Automático. A conversão usou `h264_vaapi`, CQP, QP 22 e perfil High. O app acessou as amostras por uma montagem restrita na sessão isolada. |
| Alternância de codec | ProRes exibiu os controles do card 7. Ao voltar a H.264, os valores escolhidos reapareceram e foram usados pela fila. |

Os pares de cores usados nos textos novos atingiram pelo menos 9,78:1 e APCA |Lc| 75,3, incluindo o item selecionado nos menus. Passaram a formatação, o Clippy, os testes Cargo e os quatro testes de integração com FFmpeg real.

[Painel H.264 em 880×640](capturas/velox-ajustes-h264.png) · [Ajuda em 150%](capturas/velox-ajuda-h264.png)
