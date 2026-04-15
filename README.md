# SentenceMiner

SentenceMiner e um aplicativo desktop em Tauri para capturar uma frase, gerar o verso de um flashcard com uma API compatível com OpenAI e adicionar a nota ao Anki via AnkiConnect.

O foco atual do projeto e reduzir atrito no fluxo manual de mineracao de frases. O programa nao tenta automatizar tudo o que estava previsto na especificacao original; ele implementa um fluxo mais simples que ja resolve a maior parte do problema.

## O Que O Programa Faz Hoje

O fluxo atual e este:

1. Captura uma frase selecionada na tela usando a PRIMARY selection do Linux/X11.
2. Como alternativa, roda OCR no screenshot mais recente salvo em `~/Pictures/Screenshots`.
3. Mostra a frase capturada na interface.
4. Permite que o usuario digite manualmente o termo desconhecido.
5. Gera o verso do card por API, com tres modos:
   - `iniciante`
   - `intermediario`
   - `avancado`
6. Aplica um preset simples de formatacao HTML ao termo na frente do card.
7. Envia a nota para o Anki usando o note type selecionado na UI.

Em outras palavras: hoje o app e um assistente de captura + geracao + envio ao Anki, com selecao manual do termo.

## O Que Ele Nao Faz Hoje

Estas ideias aparecem na especificacao, mas nao representam o comportamento atual do codigo:

- tokenizacao clicavel da frase;
- selecao do termo por clique em tokens;
- modo de edicao com retokenizacao;
- OCR por selecao de regiao desenhada na tela;
- fallback automatico para OCR quando nao houver selecao;
- fluxo I+1 automatizado.

Existe codigo inicial de overlay em `ui/overlay.*`, mas ele nao esta integrado ao backend atual.

## Arquitetura

### Backend

O backend fica em `src-tauri/` e expoe comandos Tauri para:

- capturar texto da PRIMARY selection;
- fazer OCR do ultimo screenshot;
- listar decks e note types do Anki;
- gerar o verso do card por HTTP;
- adicionar a nota ao Anki;
- fornecer presets e defaults para a UI.

Modulos principais:

- `src-tauri/src/main.rs`: registro dos comandos e hotkey global.
- `src-tauri/src/config.rs`: leitura e escrita de `~/.config/sentenceminer/config.toml`.
- `src-tauri/src/capture/selection.rs`: captura da selecao primaria via `arboard`.
- `src-tauri/src/capture/ocr.rs`: OCR do screenshot mais recente com `leptess`.
- `src-tauri/src/api/translation.rs`: chamada para `/chat/completions`.
- `src-tauri/src/anki/client.rs`: integracao com AnkiConnect.

### Frontend

O frontend fica em `ui/` e e HTML/CSS/JS vanilla.

A interface atual tem:

- botoes para capturar selecao e fazer OCR;
- campo da frase;
- campo manual para o termo;
- seletor do modelo de geracao;
- seletor de preset de formatacao;
- seletor de note type e deck do Anki;
- campo editavel para o verso;
- botao para adicionar ao Anki.

## Fluxo De Uso Atual

1. O usuario seleciona uma frase em outro aplicativo e usa a hotkey global, ou clica em `Capturar selecao`.
2. Se quiser, pode usar `OCR ultimo print` para extrair texto do screenshot mais recente.
3. A frase aparece no campo `Frente (frase)`.
4. O usuario digita o termo desconhecido no campo `Termo desconhecido`.
5. O usuario escolhe o modelo de geracao.
6. O usuario clica em `Gerar verso`.
7. O texto retornado pela API aparece no campo `Verso`, onde ainda pode ser editado.
8. O usuario escolhe deck e note type do Anki.
9. O usuario clica em `Adicionar ao Anki`.

Na hora de enviar, o frontend procura a primeira ocorrencia literal do termo dentro da frase e aplica o preset HTML selecionado.

## Modelos De Geracao

O backend monta prompts diferentes conforme o modelo escolhido:

- `iniciante`: traducao natural da frase e equivalente do termo em portugues.
- `intermediario`: definicao curta em portugues e ate tres sinonimos em ingles.
- `avancado`: definicao curta no idioma de origem.

O texto retornado pela API e inserido diretamente no campo `Verso`.

## Integracao Com Anki

O app usa AnkiConnect em `http://localhost:8765`.

Hoje ele:

- verifica versao/conexao;
- lista decks;
- lista note types;
- lista os campos do modelo selecionado;
- envia a nota preenchendo os dois primeiros campos do modelo com `front` e `back`.

Se nenhum deck for informado no envio, usa o deck padrao do arquivo de configuracao.

## Configuracao

Na primeira execucao o app cria:

`~/.config/sentenceminer/config.toml`

O arquivo contem:

- idiomas de origem e destino;
- host, porta, deck e tags do Anki;
- URL base, chave, modelo e timeout da API;
- hotkey global;
- idioma do OCR;
- modelo padrao da UI;
- preset padrao de formatacao;
- lista de presets HTML.

Presets padrao atuais:

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

Dependencias de sistema esperadas no Ubuntu:

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

## Limites E Observacoes

- O hotkey global pode falhar em Wayland. O app emite um aviso quando detecta esse ambiente.
- A captura de texto atual depende da PRIMARY selection; ela nao usa o clipboard comum como fallback.
- O OCR atual nao captura uma regiao da tela em tempo real. Ele le apenas o screenshot mais recente da pasta `~/Pictures/Screenshots`.
- A formatacao do termo na frente depende de correspondencia literal simples com `indexOf`, usando a primeira ocorrencia encontrada.

## Estrutura Do Repositorio

- `src-tauri/`: backend Rust/Tauri.
- `ui/`: frontend HTML/CSS/JS.
- `dev_server.py`: servidor local sem cache para a UI.
- `SentenceMiner_Spec.md`: especificacao inicial, hoje parcialmente divergente do codigo.
- `legacy-root/`: projeto antigo mantido apenas como referencia.
