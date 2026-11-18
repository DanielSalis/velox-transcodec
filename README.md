# Velox Transcode

Transcodificador nativo de vídeo em lote para Fedora. Este repositório reúne o aplicativo Rust e a página do produto.

## Instalação no Fedora Workstation

Instale as dependências de compilação e execute os comandos na raiz do projeto. [Dependências](docs/card-5-distribuicao.md)

```sh
./install.sh
~/.local/bin/velox-transcode
```

O aplicativo também aparece no menu do sistema e recebe caminhos de vídeos como argumentos. Adicione os vídeos, escolha uma pasta de destino e inicie a fila. Para desenvolvimento, use `cargo run --locked`. Os procedimentos para RPM, Flatpak, tarball e Silverblue estão na [documentação da distribuição](docs/card-5-distribuicao.md).

## Landing page

Abra [web/index.html](web/index.html) diretamente no navegador. Para testar os links de download em um checkout novo, gere os pacotes antes de servir a página. [Pré-requisitos do release](docs/card-5-distribuicao.md)

```sh
./packaging/release.sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory web
```

Na prévia local, selecione um vídeo de exemplo para inspecionar os metadados, altere o encoder ou adicione outro item e execute a fila de demonstração. Os quatro mockups mantêm estados independentes e a página carrega as fontes localmente. [Prévia local](http://127.0.0.1:8000/) · [Verificação da landing](docs/card-4-landing.md)

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
| [Distribuição](docs/card-5-distribuicao.md) | Instalação, geração de pacotes, checksums e matriz de validação |
| [Plano de desenvolvimento](docs/development-plan.md) | Cinco cards, dependências, entregas e critérios de aceite |
