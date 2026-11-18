# Velox Transcode

Transcodificador nativo de vídeo em lote para Fedora. Este repositório reúne o aplicativo Rust e a página do produto.

**Estado atual — Fundação Rust e prova de entrada nativa do card 1.**

A janela de prova recebe vários caminhos pelo arraste, seletor nativo ou argumentos. Para compilar e executá-la, consulte [dependências e evidências no Fedora](docs/card-1-fedora.md).

## Prévia da landing page

Abra [web/index.html](web/index.html) diretamente no Chrome ou Firefox. No diretório do projeto, também é possível servir a prévia localmente:

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory web
```

Acesse [a prévia local](http://127.0.0.1:8000/). Selecione um arquivo de exemplo para inspecionar os metadados, altere o encoder ou adicione outro exemplo e execute a fila de demonstração. Todos os dados de mídia e resultados do mockup são simulados. A página funciona com fontes locais.

| Documento | Referência |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Entrada para agentes, ordem de trabalho e integração |
| [PRODUCT.md](PRODUCT.md) | Objetivo, experiência e escopo aprovado em cada fase |
| [DESIGN.md](DESIGN.md) | Brand kit, tokens, contraste, identidade e composição |
| [Arquitetura](docs/architecture.md) | Módulos, interfaces e decisões do motor, desktop e web |
| [Prova no Fedora](docs/card-1-fedora.md) | Dependências, testes e observações do card 1 |
| [Plano de desenvolvimento](docs/development-plan.md) | Cinco cards, dependências, entregas e critérios de aceite |
