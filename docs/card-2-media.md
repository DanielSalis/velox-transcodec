# Card 2 — Motor FFmpeg e fila

O motor executa ferramentas externas a partir do `PATH`. O build do FFmpeg precisa expor e conseguir inicializar pelo menos um encoder H.264 para que a conversão esteja disponível. `libx264` é a opção de fallback quando nenhum alvo de hardware passa na sondagem. AAC precisa estar disponível no FFmpeg quando a conversão de áudio é selecionada.

## Integração no card 3

No runtime Tokio, chame `queue::spawn(MediaEngine::default())` e consuma continuamente o receptor `events`. Envie comandos por `commands`.

1. Envie `DetectEncoders`. Mostre apenas capacidades com `support.is_ok()`. A primeira suportada é a sugestão inicial. Leia `quality_mode`, `presets` e `profiles` do alvo escolhido.
2. Envie `AddFiles` com os caminhos. `ItemAdded` atribui o ID e `MediaProbed` confirma que a entrada está pronta. Preserve o ID para relacionar eventos que podem chegar fora da ordem de adição.
3. Envie `StartBatch` com os IDs prontos e uma cópia de `OutputSettings`. Use o `BatchId` de `BatchStarted` ao enviar `CancelBatch`. O mesmo ID de mídia pode ser reutilizado em outro lote. IDs repetidos na mesma seleção são tratados uma vez.
4. Use `BatchFinished` para o resumo e a notificação. Os estados terminais dos itens já chegaram antes desse evento. Progresso intermediário pode ser coalescido. `JobOutput` fornece o caminho publicado, os bytes e a redução em relação à entrada.
5. Em `ItemState::Failed` com `JobFailureKind::AudioCopyIncompatible`, ofereça AAC e permita tentar de novo. Uma falha de encoder durante o lote também é reportada por item, preservando a seleção e os resultados das outras entradas.

Os percentuais estão na faixa 0–100. Quando falta duração em algum item, o progresso total usa a contagem de itens encerrados. O ETA do lote só é publicado quando há estimativas para todos os itens restantes. Ao fechar a janela, encerrar o receptor libera a fila e cancela o lote. [Contratos de fila](../src/domain/queue.rs) · [API da fila](../src/queue/mod.rs)

## Evidências no Fedora

Fedora Workstation 44 x86_64, Ryzen 7 6800H com 16 CPUs lógicas, executáveis `ffmpeg-free`/ffprobe 8.0.1 com `libavcodec-freeworld`.

| Alvo | Resultado |
| --- | --- |
| NVIDIA GeForce RTX 3070 Ti Laptop, NVENC GPU 0 | Sondagem de capacidade e transcode H.264 real em MP4. |
| AMD Radeon integrada, VA-API em `renderD128` | O driver não apresentou perfil H.264 de encode utilizável. |
| NVIDIA, VA-API em `renderD129` | O dispositivo VA-API falhou ao ser inicializado. O caminho NVENC passou no mesmo host. |
| CPU, libx264 | Saídas MP4 com Copy, MKV com AAC 192 kbps e MOV com AAC 320 kbps, inspecionadas pelo ffprobe. Dois jobs x264 com mesmo nome de origem terminaram sem sobrescrever a saída pré-existente. |

A integração também verificou mídia inválida, caminhos com acento e espaço, rejeição de alvo indisponível, Copy incompatível com MP4 e recuperação por AAC, mídia alterada depois do probe e cancelamento real com limpeza de temporários. O sucesso de VA-API continua a validar em um host cujo driver exponha encode H.264. A sondagem testa uma amostra de 640×360, e alguns encoders podem ter restrições adicionais para dimensões e formatos de entrada.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --test media_pipeline -- --ignored --test-threads=1
```

### Amostra de desempenho x264

Uma entrada sintética de oito segundos, 1080p a 30 FPS, convertida com CRF 22, preset medium e oito threads de encode, reportou **98 FPS** no último bloco de progresso, **2,57 s** de tempo de processo e **713.212 KiB** de RSS máximo do FFmpeg. O arquivo passou de 13.070.982 para 6.650.395 bytes. A medição usa um único subprocesso com os parâmetros da fila. A coleta de metadados e a criação da entrada ocorrem fora do cronômetro.

```sh
qa="$(mktemp -d)"
ffmpeg -v error -nostdin -f lavfi -i 'testsrc2=s=1920x1080:r=30' \
  -f lavfi -i 'sine=f=1000:sample_rate=48000' -t 8 \
  -c:v mpeg4 -q:v 6 -threads:v 2 -c:a aac "$qa/entrada.mp4"
/usr/bin/time -f 'wall_s=%e peak_rss_kib=%M' ffmpeg -hide_banner -v error \
  -nostdin -nostats -xerror -abort_on empty_output_stream \
  -progress pipe:1 -stats_period 0.25 -y -filter_threads 2 -threads 2 \
  -protocol_whitelist file -i "$qa/entrada.mp4" -map 0:0 -map '0:a?' \
  -c:v libx264 -profile:v high -preset medium -vf 'pad=ceil(iw/2)*2:ceil(ih/2)*2' \
  -pix_fmt yuv420p -crf 22 -threads:v 8 -c:a copy -movflags +faststart -f mp4 "$qa/saida.mp4"
```
