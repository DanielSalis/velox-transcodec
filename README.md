# Velox Transcode

Transcodificador nativo de vídeo em lote para Fedora. Este repositório reúne o aplicativo Rust e a página do produto.

**Estado atual — Aplicação nativa e landing page implementadas.**

No Fedora, use as [dependências de compilação da base Rust](docs/card-1-fedora.md) e execute:

```sh
cargo run --locked
```

Adicione os vídeos, escolha uma pasta de destino e inicie a fila. O binário também recebe caminhos locais como argumentos. As [evidências do desktop](docs/card-3-desktop.md) registram o fluxo e os ambientes testados.

## Landing page

Abra [web/index.html](web/index.html) diretamente no Chrome ou Firefox. No diretório do projeto, também é possível servir a prévia localmente:

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory web
```

Acesse [a prévia local](http://127.0.0.1:8000/). Selecione um arquivo de exemplo para inspecionar os metadados, altere o encoder ou adicione outro exemplo e execute a fila de demonstração. Os quatro mockups usam arquivos de exemplo e mantêm estados independentes. A página funciona com fontes locais. Os canais de instalação em preparação estão identificados. [Verificação da landing](docs/card-4-landing.md)

| Documento | Referência |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Entrada para agentes, ordem de trabalho e integração |
| [PRODUCT.md](PRODUCT.md) | Objetivo, experiência e escopo aprovado em cada fase |
| [DESIGN.md](DESIGN.md) | Brand kit, tokens, contraste, identidade e composição |
| [Arquitetura](docs/architecture.md) | Módulos, interfaces e decisões do motor, desktop e web |
| [Prova no Fedora](docs/card-1-fedora.md) | Dependências, testes e observações do card 1 |
| [Motor e fila](docs/card-2-media.md) | Integração Rust, testes reais e medição do card 2 |
| [Aplicação desktop](docs/card-3-desktop.md) | Fluxo nativo, captura e resultados do card 3 |
| [Landing page](docs/card-4-landing.md) | Mockups, verificação responsiva e canais de instalação |
| [Plano de desenvolvimento](docs/development-plan.md) | Cinco cards, dependências, entregas e critérios de aceite |
