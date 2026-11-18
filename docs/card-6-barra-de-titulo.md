# Card 6 — Barra de título do Velox

O cabeçalho do Velox é a barra de título da janela. A área com a marca permite arrastar e dar duplo clique para maximizar ou restaurar. Os controles à direita têm foco visível, rótulos em português e acionamento por Enter ou Espaço. Maximizar e restaurar acompanham o estado do compositor.

O layout ocupa toda a área da janela, com linhas de 1 px entre a barra de título, os painéis e a fila.

O Slint controla os estados da janela e encaminha Fechar ao pedido de fechamento existente. No backend Qt, uma ponte C++ recebe as bordas acionadas e chama `QWindow::startSystemResize` com a janela nativa do Slint. O compositor faz o redimensionamento e aplica o tamanho mínimo. As áreas de borda ficam inativas enquanto a janela está maximizada. [Janela Slint](https://docs.slint.dev/latest/docs/slint/reference/window/window/) · [Redimensionamento interativo no Qt](https://doc.qt.io/qt-6.8/qwindow.html#startSystemResize)

## Verificação

| Ambiente | Resultado |
| --- | --- |
| Fedora 44, Qt 6.11.2, Weston 15.0.1 e Wayland | Arrastar e redimensionar alteraram a geometria. O trace confirmou `move`, `resize`, `set_maximized`, `unset_maximized` e `set_minimized`, além do app ID `dev.veloxtranscode.Velox`. Duplo clique e restauração por teclado passaram. Com escala Qt de 150%, maximizar, restaurar e fechar funcionaram. |
| Fedora 44, Openbox 3.6.1 e X11 no Xvfb | Minimizar, maximizar, restaurar e fechar funcionaram por mouse e teclado. O gerenciador de janelas também restaurou a janela maximizada. O redimensionamento respeitou 880×640 px lógicos. A interface foi verificada a 100% e 150%. `WM_CLASS` permaneceu `velox-transcode`. |
| Release instalado em prefixo de teste | O arquivo `.desktop` abriu o binário instalado com a classe esperada. O atalho Alt+F4 do Openbox fechou a janela. |
| Arraste externo em Wayland e X11 | Uma janela GTK entregou dois vídeos em uma operação. As duas linhas chegaram ao estado **Pronto**, com metadados de MPEG4 e VP9. |
| Lote ativo em X11 | Com FFmpeg real em execução, minimizar manteve o processo de conversão ativo. Após reabrir a janela, clicar em Fechar encerrou o FFmpeg e removeu o arquivo temporário antes de sair. |

Passaram `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, os dois testes de integração que exigem FFmpeg real e `cargo build --locked --release`.

[Janela restaurada em X11](capturas/velox-barra-restaurada.png) · [Janela maximizada em Wayland](capturas/velox-barra-maximizada.png)
