# Velox Transcode

Transcodificador nativo de vídeo em lote para Fedora. Este repositório reúne o aplicativo Rust e a página do produto.

**Estado atual — Fase 1, aguardando aprovação visual e dos cinco cards.**

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
| [Plano de desenvolvimento](docs/development-plan.md) | Cinco cards, dependências, entregas e critérios de aceite |
