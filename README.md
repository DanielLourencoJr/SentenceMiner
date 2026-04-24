# SentenceMiner

SentenceMiner é um aplicativo desktop em Tauri para capturar uma frase, gerar o verso de um flashcard com uma API compatível com OpenAI e adicionar a nota ao Anki via AnkiConnect.

O foco atual do projeto é reduzir atrito no fluxo manual de mineração de frases. O programa não tenta automatizar tudo o que estava previsto na especificação original; ele implementa um fluxo mais simples que já resolve a maior parte do problema.

## O Que O Programa Faz Hoje

O fluxo atual é este:

1. Captura uma frase selecionada na tela usando a PRIMARY selection do Linux/X11.
2. Como alternativa, roda OCR no screenshot mais recente salvo em `~/Pictures/Screenshots`.
3. Mostra a frase capturada na interface.
4. Permite que o usuário digite manualmente o termo desconhecido.
5. Gera o verso do card por API, com três modos:
   - `iniciante`
   - `intermediario`
   - `avancado`
6. Aplica um preset simples de formatação HTML ao termo na frente do card.
7. Envia a nota para o Anki usando o note type selecionado na UI.

Em outras palavras: hoje o app é um assistente de captura + geração + envio ao Anki, com seleção manual do termo.

## O Que Ele Não Faz Hoje

Estas ideias aparecem na especificação, mas não representam o comportamento atual do código:

- tokenização clicável da frase;
- seleção do termo por clique em tokens;
- modo de edição com retokenização;
- OCR por seleção de região desenhada na tela;
- fallback automático para OCR quando não houver seleção;
- gerenciamento de vocabulário conhecido.

Existe código inicial de overlay em `ui/overlay.*`, mas ele não está integrado ao backend atual.

## Arquitetura

### Backend

O backend fica em `src-tauri/` e expõe comandos Tauri para:

- capturar texto da PRIMARY selection;
- fazer OCR do último screenshot;
- listar decks e note types do Anki;
- gerar o verso do card por HTTP;
- adicionar a nota ao Anki;
- fornecer presets e defaults para a UI.

Módulos principais:

- `src-tauri/src/main.rs`: registro dos comandos e hotkey global.
- `src-tauri/src/config.rs`: leitura e escrita de `~/.config/sentenceminer/config.toml`.
- `src-tauri/src/capture/selection.rs`: captura da seleção primária via `arboard`.
- `src-tauri/src/capture/ocr.rs`: OCR do screenshot mais recente com `leptess`.
- `src-tauri/src/api/translation.rs`: chamada para `/chat/completions`.
- `src-tauri/src/anki/client.rs`: integração com AnkiConnect.

### Frontend

O frontend fica em `ui/` e é HTML/CSS/JS vanilla.

A interface atual tem:

- botões para capturar seleção e fazer OCR;
- campo da frase;
- campo manual para o termo;
- seletor do modelo de geração;
- seletor de preset de formatação;
- seletor de note type e deck do Anki;
- campo editável para o verso;
- botão para adicionar ao Anki.

## Fluxo De Uso Atual

1. O usuário seleciona uma frase em outro aplicativo e usa a hotkey global, ou clica em `Capturar selecao`.
2. Se quiser, pode usar `OCR ultimo print` para extrair texto do screenshot mais recente.
3. A frase aparece no campo `Frente (frase)`.
4. O usuário digita o termo desconhecido no campo `Termo desconhecido`.
5. O usuário escolhe o modelo de geração.
6. O usuário clica em `Gerar verso`.
7. O texto retornado pela API aparece no campo `Verso`, onde ainda pode ser editado.
8. O usuário escolhe deck e note type do Anki.
9. O usuário clica em `Adicionar ao Anki`.

Na hora de enviar, o frontend procura a primeira ocorrência literal do termo dentro da frase e aplica o preset HTML selecionado.

## Modelos De Geração

O backend monta prompts diferentes conforme o modelo escolhido:

- `iniciante`: tradução natural da frase e equivalente do termo em português.
- `intermediario`: definição curta em português e até três sinônimos em inglês.
- `avancado`: definição curta no idioma de origem.

O texto retornado pela API é inserido diretamente no campo `Verso`.

## Integração Com Anki

O app usa AnkiConnect em `http://localhost:8765`.

Hoje ele:

- verifica versão/conexão;
- lista decks;
- lista note types;
- lista os campos do modelo selecionado;
- envia a nota preenchendo os dois primeiros campos do modelo com `front` e `back`.

Se nenhum deck for informado no envio, usa o deck padrão do arquivo de configuração.

## Configuração

Na primeira execução o app cria:

`~/.config/sentenceminer/config.toml`

O arquivo contém:

- idiomas de origem e destino;
- host, porta, deck e tags do Anki;
- URL base, chave, modelo e timeout da API;
- hotkey global;
- idioma do OCR;
- modelo padrão da UI;
- preset padrão de formatação;
- lista de presets HTML.

Presets padrão atuais:

- `negrito`
- `laranja`
- `sublinhado`

## Desenvolvimento

### Requisitos

- Rust 2021
- Tauri v2
- Anki com AnkiConnect
- Tesseract e Leptonica
- ambiente Linux com suporte adequado para PRIMARY selection

Dependências de sistema esperadas no Ubuntu:

```bash
sudo apt install \
  tesseract-ocr \
  tesseract-ocr-eng \
  libtesseract-dev \
  libleptonica-dev \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev
```

### Rodar Em Desenvolvimento

Em um terminal, servir a UI sem cache:

```bash
python3 dev_server.py
```

Em outro terminal, iniciar o app Tauri:

```bash
cd src-tauri
cargo tauri dev
```

Se `/tmp` for pequeno:

```bash
export TMPDIR=/media/<disk>/tmp
export CARGO_TARGET_DIR=/media/<disk>/sentenceminer-target
```

## Limites E Observações

- O hotkey global pode falhar em Wayland. O app emite um aviso quando detecta esse ambiente.
- A captura de texto atual depende da PRIMARY selection; ela não usa o clipboard comum como fallback.
- O OCR atual não captura uma região da tela em tempo real. Ele lê apenas o screenshot mais recente da pasta `~/Pictures/Screenshots`.
- A formatação do termo na frente depende de correspondência literal simples com `indexOf`, usando a primeira ocorrência encontrada.

## Estrutura Do Repositório

- `src-tauri/`: backend Rust/Tauri.
- `ui/`: frontend HTML/CSS/JS.
- `dev_server.py`: servidor local sem cache para a UI.
- `SentenceMiner_Spec.md`: especificação inicial, hoje parcialmente divergente do código.
- `legacy-root/`: projeto antigo mantido apenas como referência.
