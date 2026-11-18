# Card 7 — Apple ProRes

O seletor no topo do painel **Saída** alterna entre H.264 e **ProRes**. Para edição, o Velox gera MOV com ProRes 4:2:2 de 10 bits pela CPU. Selecione **422 Proxy**, **422 LT**, **422** ou **422 HQ**. O perfil inicial é **422** quando disponível. Os arquivos recebem o sufixo `-prores.mov` e podem ficar maiores que os originais. [Perfis no FFmpeg](https://ffmpeg.org/ffmpeg-codecs.html#ProRes)

**PCM 24 bits** é a opção inicial de áudio. **Copiar** preserva o áudio de origem quando o MOV pode armazená-lo. Em caso de incompatibilidade, selecione o item com falha e use **Usar PCM 24 bits** para preparar uma nova tentativa. A volta ao H.264 recupera os ajustes escolhidos nessa sessão.

O Velox procura o encoder `prores_ks` no FFmpeg e testa cada perfil antes de oferecê-lo. O requisito vale para o FFmpeg instalado no Fedora ou no Arch, e para o FFmpeg do runtime KDE no Flatpak.

## Verificação

| Ambiente | Evidência |
| --- | --- |
| Fedora 44 e FFmpeg 8.0.1 | O teste da fila confirmou os quatro perfis, MOV com marca QuickTime, vídeo `yuv422p10le`, PCM de 24 bits, cópia de AAC e vídeo sem áudio. O cenário sem `prores_ks` rejeitou o lote. Áudio TTA incompatível com MOV pôde ser convertido para PCM. Cancelamento removeu os temporários e a colisão de nome preservou a saída anterior. |
| Interface nativa no Fedora, binário instalado do tarball | Foram gerados os quatro perfis pela interface. A troca de ProRes para H.264 recuperou encoder x264, CRF 23, preset veryfast, perfil Main, AAC 192 kbps e MOV. A ação **Usar PCM 24 bits** permitiu repetir com sucesso a entrada com áudio TTA. |
| Bundle Flatpak, KDE Platform 6.11 | A versão empacotada gerou os quatro perfis pela interface, com PCM em três saídas e AAC copiado na saída LT. A sessão de teste concedeu acesso à pasta das amostras e montou o caminho do portal de documentos isolado para escrita. |
| RPM no Fedora 44 | O DNF instalou o RPM atualizado. O desktop entry passou na validação, o FFmpeg padrão expôs `prores_ks` e o aplicativo iniciou com Qt offscreen. |
| Tarball no Arch Linux | A instalação por usuário comum e a inicialização Qt offscreen passaram em contêiner. O FFmpeg do Arch expôs `prores_ks`. |
| Landing e pacotes locais | A página informa as duas opções de saída. A conferência em Chromium cobriu 1440 e 390 px, navegação por teclado, contraste e comandos de instalação. Os três artefatos locais conferem com o manifesto SHA-256. |

Passaram `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` e `cargo test --locked --test media_pipeline -- --ignored`. Os testes H.264 incluem MP4, MKV e MOV.

[Captura do painel ProRes](capturas/velox-prores.png) · [Checksums dos pacotes locais](../web/downloads/SHA256SUMS)
