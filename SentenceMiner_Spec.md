# SentenceMiner — Documento de Especificação Técnica
**v1.1 — Uso pessoal, Ubuntu Desktop**

---

## Índice

1. [Contexto e Objetivo](#1-contexto-e-objetivo)
2. [Stack Tecnológica](#2-stack-tecnológica)
3. [Arquitetura](#3-arquitetura)
4. [Fluxo de Uso](#4-fluxo-de-uso)
5. [Features e Requisitos Funcionais](#5-features-e-requisitos-funcionais)
6. [Modelos de Card](#6-modelos-de-card)
7. [Integração AnkiConnect](#7-integração-ankiconnect)
8. [Integração API de Tradução](#8-integração-api-de-tradução)
9. [Configurações](#9-configurações)
10. [Estrutura de Diretórios](#10-estrutura-de-diretórios)
11. [Requisitos Não-Funcionais](#11-requisitos-não-funcionais)
12. [Plano de Implementação](#12-plano-de-implementação)
13. [Instruções para o Agente](#13-instruções-para-o-agente)

---

## 1. Contexto e Objetivo

SentenceMiner é um aplicativo desktop para Ubuntu que assiste ao estudo de inglês pelo método de frases com Anki. O programa captura texto da tela, apresenta a frase ao usuário como tokens clicáveis, e após o usuário marcar o(s) termo(s) desconhecido(s), gera o verso do flashcard via API e o envia ao Anki.

O programa **não gerencia vocabulário**. Ele não tenta inferir o que o usuário sabe ou não sabe. Toda decisão sobre qual termo é desconhecido é do usuário.

### Método I+1 (referência conceitual)

O método I+1 (Krashen) postula que a aquisição de língua acontece com input compreensível que contém exatamente uma unidade desconhecida. O SentenceMiner não implementa scoring automático de I+1 — ele apenas facilita que o usuário extraia frases desse tipo de qualquer fonte e as converta em flashcards.

---

## 2. Stack Tecnológica

| Camada | Tecnologia | Justificativa |
|---|---|---|
| Linguagem | Rust (edição 2021) | Performance, baixo uso de memória, sem GC, preferência do usuário |
| GUI | Tauri v2 | Webview nativa leve, UI em HTML/CSS/JS, lógica em Rust, sem Electron |
| Frontend UI | HTML + CSS + JavaScript vanilla | Sem frameworks — manter simples e leve |
| Captura de texto | `arboard` crate | Leitura da PRIMARY selection (X11) |
| OCR | Tesseract via `leptess` crate | OCR local, sem rede, suporte a inglês (extensível) |
| Sobreposição OCR | Janela Tauri secundária fullscreen transparente | Seleção de região para OCR |
| Screenshot | `screenshots` crate | Captura da região selecionada |
| Hotkey global | `global-hotkey` crate (plugin Tauri) | Listener de hotkey independente do foco |
| HTTP client | `reqwest` com `tokio` | Chamadas assíncronas à API de tradução e AnkiConnect |
| Serialização | `serde` + `serde_json` | JSON para comunicação com APIs |
| Configuração | `toml` crate + `~/.config/sentenceminer/config.toml` | Configuração persistida e editável |
| Build | `cargo` + `tauri-cli` | Build padrão do ecossistema Tauri |

### Dependências do sistema (apt)

```
tesseract-ocr
tesseract-ocr-eng
libtesseract-dev
libleptonica-dev
libwebkit2gtk-4.1-dev    (Tauri v2)
libappindicator3-dev
librsvg2-dev
```

> **Wayland vs X11:** Ubuntu 22.04+ usa Wayland com XWayland. A leitura de PRIMARY selection via `arboard` pode requerer fallback para `xdotool getactivewindow` em alguns ambientes Wayland. Testar e documentar o comportamento na Fase 1.

---

## 3. Arquitetura

O programa é uma aplicação Tauri com janela principal sempre aberta. A lógica de captura, OCR, chamadas de API e AnkiConnect rodam no backend Rust. A janela é o frontend HTML/JS que se comunica com o backend via comandos Tauri (`invoke`).

```
[Frontend — HTML/JS]
   |  invoke("capture_selection")
   |  invoke("capture_ocr_region", { x, y, w, h })
   |  invoke("generate_back", { sentence, terms, model })
   |  invoke("add_to_anki", { front, back, deck, tags, format_preset })
   v
[Backend Rust]
   ├── capture::selection     — X11 PRIMARY selection via arboard
   ├── capture::ocr           — Screenshot de região + Tesseract
   ├── api::translation       — OpenAI-compatible HTTP client
   ├── anki::client           — AnkiConnect HTTP client
   └── config                 — Leitura/escrita de config.toml
```

### Comunicação frontend ↔ backend

Usar exclusivamente o sistema de comandos do Tauri (`#[tauri::command]` no Rust, `invoke()` no JS). Não usar `eval` nem manipulação direta de estado fora desse canal.

---

## 4. Fluxo de Uso

Este é o fluxo canônico completo. O agente deve implementá-lo exatamente como descrito.

```
1. Usuário está lendo conteúdo em inglês em qualquer aplicativo.

2. Usuário seleciona uma frase com o mouse.

3. Usuário pressiona o hotkey global (padrão: Ctrl+Shift+S).

4. Backend lê a X11 PRIMARY selection.
   - Se vazia: abre janela de sobreposição OCR (ver F2).
   - Se OCR também falhar: exibir mensagem de erro na janela.

5. O texto capturado é tokenizado por espaços e pontuação e exibido
   na área "Frente" como uma sequência de tokens clicáveis (ver F3).

6. O usuário clica nos tokens que formam o termo desconhecido.
   - Tokens clicados ficam visualmente destacados.
   - Múltiplos tokens podem ser selecionados (termos compostos ou não contíguos).
   - Clicar num token já selecionado deseleciona.

7. O usuário escolhe o modelo de card (Iniciante / Intermediário / Avançado)
   e o preset de formatação.

8. O usuário clica em "Gerar".
   - Backend monta o prompt com a frase e a concatenação dos tokens selecionados.
   - Chamada assíncrona à API. Frontend exibe estado de loading.

9. O verso gerado aparece no campo "Verso" (editável).

10. O usuário pode editar o verso livremente.

11. O usuário clica em "Adicionar ao Anki".
    - Backend aplica a formatação do preset ao termo na frente.
    - Backend envia o card via AnkiConnect.
    - Exibir confirmação de sucesso ou mensagem de erro.

12. Botão "Limpar" reseta os campos para a próxima captura.
    Os campos não são limpos automaticamente após envio.
```

### Comportamentos de erro

| Situação | Comportamento |
|---|---|
| Hotkey acionado sem seleção e OCR retorna vazio | Mensagem na janela: "Nenhum texto capturado." |
| Usuário clica "Gerar" sem tokens selecionados | Mensagem: "Selecione o termo desconhecido antes de gerar." |
| Falha na API de tradução | Mensagem de erro com status HTTP. Verso permanece editável para preenchimento manual. |
| Timeout na API | Mensagem: "A API não respondeu a tempo. Preencha o verso manualmente." |
| Anki não está rodando | Mensagem: "Não foi possível conectar ao AnkiConnect. O Anki está aberto?" |
| AnkiConnect retorna erro | Exibir a mensagem de erro retornada pela API. |

---

## 5. Features e Requisitos Funcionais

### F1 — Captura por seleção de texto

- Ler X11 PRIMARY selection via `arboard` quando o hotkey é pressionado.
- Se vazia: acionar F2 diretamente. Não tentar CLIPBOARD como fallback.

### F2 — Captura por OCR com seleção de região

- Ao ser acionado, abrir uma janela Tauri secundária fullscreen, transparente e sem decorações, com cursor crosshair.
- O usuário arrasta para selecionar uma região retangular.
- Ao soltar o mouse: fechar a sobreposição, capturar screenshot da região selecionada, processar com Tesseract.
- Pré-processamento da imagem antes do OCR: upscale 2x, binarização. Usar `image` crate.
- Retornar o texto extraído. Se vazio, reportar erro.

### F3 — Área "Frente" com tokens clicáveis

- O texto capturado é tokenizado no backend por espaços e pontuação. Cada token é enviado ao frontend como um array de strings.
- O frontend renderiza cada token como um elemento clicável (ex: `<span class="token">`).
- Estado dos tokens: normal | selecionado. Toggle ao clicar.
- Tokens selecionados recebem destaque visual claro (ex: fundo colorido).
- A pontuação adjacente a palavras (vírgulas, pontos) é renderizada como token não clicável, apenas visual.
- **Modo de edição:** um ícone de lápis ao lado da área converte os tokens em um `<textarea>` editável com o texto completo. Clicar no ícone novamente retokeniza o texto editado e volta ao modo de tokens. O modo padrão é tokens clicáveis.

### F4 — Seleção de modelo e preset

- Seletor de modelo: três opções fixas (Iniciante / Intermediário / Avançado). Padrão configurável.
- Seletor de preset de formatação: lista os presets definidos no config. Padrão configurável.
- Ambos visíveis e alteráveis antes de clicar "Gerar".

### F5 — Geração do verso via API

- Ver Seção 8.
- Chamada assíncrona — a UI não trava.
- O verso é inserido no campo editável quando a resposta chega.

### F6 — Envio ao Anki

- Ver Seção 7.
- Antes de enviar, o backend aplica a formatação do preset selecionado ao(s) token(s) marcados na frente.
- Após sucesso: mensagem de confirmação na área de status. Campos não são limpos automaticamente.

---

## 6. Modelos de Card

### Frente do card

A frase completa com o termo alvo formatado conforme o preset selecionado (ver Seção 9). A formatação é aplicada pelo programa no momento do envio ao Anki — nunca pelo modelo de linguagem.

Exemplo com preset "negrito":
```
She looked at him with an <b>inscrutable</b> expression.
```

Exemplo com preset "laranja":
```
She looked at him with an <span style="color: #e05c00">inscrutable</span> expression.
```

---

### Modelo Iniciante

**Objetivo:** tradução completa da frase em português + equivalente do termo desconhecido em português, sem formatação — o programa localiza o equivalente e aplica a formatação por conta própria.

**Prompt para a API:**

```
You are a language flashcard assistant.

Source language: {source_language}
Target language: {target_language}
Sentence: "{sentence}"
Unknown term: "{term}"

Generate the back of a flashcard. Respond in {target_language}. Include:
1. A natural translation of the full sentence.
2. On a new line, only the {target_language} equivalent of the unknown term — nothing else.

Respond only with the flashcard content. No labels, no preamble, no formatting.
```

**Exemplo de verso esperado (saída bruta da API):**

```
Ela o olhou com uma expressão inescrutável.
inescrutável
```

O programa usa a segunda linha para localizar o equivalente na tradução e aplica o preset de formatação a ele.

---

### Modelo Intermediário

**Objetivo:** definição em português do termo desconhecido + sinônimos em inglês.

**Prompt para a API:**

```
You are a language flashcard assistant.

Source language: {source_language}
Target language: {target_language}
Sentence: "{sentence}"
Unknown term: "{term}"

Generate the back of a flashcard. Respond in {target_language}. Include:
1. A concise definition of the unknown term in {target_language}.
2. Up to three synonyms in {source_language}.

Respond only with the flashcard content. No labels, no preamble, no formatting.
```

**Exemplo de verso esperado:**

```
Difícil de entender ou interpretar; que não revela sentimentos ou intenções.

Sinônimos: enigmatic, impenetrable, opaque
```

---

### Modelo Avançado

**Objetivo:** definição do termo no idioma-alvo (inglês), sem português.

**Prompt para a API:**

```
You are a language flashcard assistant.

Source language: {source_language}
Sentence: "{sentence}"
Unknown term: "{term}"

Generate the back of a flashcard entirely in {source_language}. Include only a concise definition of the unknown term.

Respond only with the flashcard content. No labels, no preamble, no formatting.
```

**Exemplo de verso esperado:**

```
Impossible to understand or interpret; not readily giving away thoughts or feelings.
```

---

## 7. Integração AnkiConnect

O Anki deve estar rodando localmente com o add-on **AnkiConnect** instalado. Ele expõe uma API REST em `http://localhost:8765`.

### Note type usado

O programa usa o note type padrão **Basic** do Anki, com os campos `Front` e `Back`. Não criar note types customizados.

### Chamada para adicionar card

```json
POST http://localhost:8765
Content-Type: application/json

{
  "action": "addNote",
  "version": 6,
  "params": {
    "note": {
      "deckName": "<deck configurado>",
      "modelName": "Basic",
      "fields": {
        "Front": "<frase com termo formatado conforme preset>",
        "Back": "<verso gerado>"
      },
      "tags": ["sentenceminer"]
    }
  }
}
```

### Verificação de conexão

Na inicialização, fazer chamada de `version`:

```json
{ "action": "version", "version": 6 }
```

Se falhar, exibir aviso na área de status. O programa continua funcional para captura e geração.

### Listagem de decks

No diálogo de configurações, buscar decks via:

```json
{ "action": "deckNames", "version": 6 }
```

Apresentar como lista selecionável para o usuário definir o deck padrão.

---

## 8. Integração API de Tradução

A API segue o padrão OpenAI (`/v1/chat/completions`). A implementação deve funcionar com qualquer endpoint OpenAI-compatible. O usuário usará Groq.

### Configuração

Definida no `config.toml`:
- `api_base_url`
- `api_key` — nunca logar ou exibir na UI
- `model`

### Formato da chamada

```json
POST {api_base_url}/chat/completions
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "model": "{model}",
  "messages": [
    { "role": "user", "content": "{prompt_do_modelo_escolhido}" }
  ],
  "temperature": 0.3,
  "max_tokens": 300
}
```

### Resposta

Extrair `choices[0].message.content` e inserir no campo "Verso".

### Timeout

15 segundos. Se exceder, exibir erro e deixar o verso editável para preenchimento manual.

---

## 9. Configurações

Arquivo: `~/.config/sentenceminer/config.toml`

Criado automaticamente com valores padrão na primeira execução.

```toml
[general]
source_language = "English"
target_language = "Brazilian Portuguese"

[anki]
host = "localhost"
port = 8765
deck = "Default"
tags = ["sentenceminer"]

[api]
base_url = "https://api.groq.com/openai/v1"
api_key = ""
model = "llama3-70b-8192"
timeout_seconds = 15

[capture]
hotkey = "ctrl+shift+s"
ocr_language = "eng"

[ui]
default_model = "intermediario"      # iniciante | intermediario | avancado
default_format_preset = "negrito"

[[format_presets]]
name = "negrito"
template = "<b>{term}</b>"

[[format_presets]]
name = "laranja"
template = "<span style=\"color: #e05c00\">{term}</span>"

[[format_presets]]
name = "sublinhado"
template = "<u>{term}</u>"
```

### Presets de formatação

Cada preset tem um `name` (exibido na UI) e um `template` com `{term}` como placeholder. O usuário pode adicionar quantos presets quiser editando o TOML diretamente. A UI lista os presets disponíveis num seletor — não há editor de presets na UI.

---

## 10. Estrutura de Diretórios

```
sentenceminer/
├── Cargo.toml
├── Cargo.lock
├── tauri.conf.json
├── README.md
├── install.sh
├── src/                          # Backend Rust
│   ├── main.rs                   # Entry point Tauri, registro de comandos e hotkey
│   ├── config.rs                 # Lê/escreve config.toml, structs de configuração
│   ├── capture/
│   │   ├── mod.rs
│   │   ├── selection.rs          # X11 PRIMARY selection via arboard
│   │   └── ocr.rs                # Screenshot de região + Tesseract
│   ├── api/
│   │   ├── mod.rs
│   │   └── translation.rs        # OpenAI-compatible client, montagem de prompts
│   └── anki/
│       ├── mod.rs
│       └── client.rs             # AnkiConnect HTTP client
└── ui/                           # Frontend
    ├── index.html
    ├── style.css
    └── main.js                   # invoke() calls, lógica de tokens, estado da UI
```

---

## 11. Requisitos Não-Funcionais

| Requisito | Meta |
|---|---|
| Captura (seleção → tokens na tela) | < 200ms |
| OCR (região selecionada) | < 4s |
| Uso de memória em idle | < 80MB RAM |
| Tempo de inicialização | < 1.5s |
| Privacidade | Zero dados enviados para fora exceto chamadas explícitas à API configurada pelo usuário |
| Responsividade da UI | Nenhuma operação de I/O bloqueia a UI. Toda rede e OCR rodam em tasks assíncronas Rust. |
| Crashes | Nenhum `panic` não tratado termina o processo silenciosamente. Usar `Result` em toda a lógica de negócio. |

---

## 12. Plano de Implementação

Implementar na ordem abaixo. Não avançar para a próxima fase sem verificar a atual.

### Fase 1 — Fundação

**Status:** Concluída (2026-04-10).

1. Inicializar projeto Tauri v2: `cargo tauri init`
2. Implementar `config.rs`: leitura/escrita do `config.toml`, criação do arquivo padrão com todos os campos se ausente
3. Implementar `capture/selection.rs`: ler X11 PRIMARY selection via `arboard`
4. Expor como comando Tauri `capture_selection` e testar invocando do frontend via `invoke`

**Verificação:** selecionar texto em qualquer app, clicar num botão de teste no frontend, ver o texto retornado no console do browser (DevTools do Tauri).

---

### Fase 2 — OCR

**Status:** Concluída com alteração de fluxo (2026-04-10).  
**Nota:** OCR agora usa o último screenshot em `~/Pictures/Screenshots` em vez de overlay/seleção de região.

1. Implementar `capture/ocr.rs`: janela de sobreposição fullscreen, seleção de região, screenshot, Tesseract
2. Expor como comando `capture_ocr_region`
3. Integrar com o fluxo: se `capture_selection` retornar vazio, chamar fluxo OCR

**Verificação:** com nada selecionado, acionar hotkey, selecionar região com texto, ver texto extraído no frontend.

---

### Fase 3 — AnkiConnect

**Status:** Concluída (2026-04-10).  
**Nota:** Seleção de note type via `modelNames`; campos são mapeados pelo `modelFieldNames`.

1. Implementar `anki/client.rs`: `check_connection()`, `get_deck_names()`, `add_note()`
2. Expor como comandos Tauri
3. Verificar conexão na inicialização e exibir status no frontend

**Verificação:** com Anki aberto, chamar `add_note` com dados hardcoded, confirmar card no browser do Anki.

---

### Fase 4 — API de tradução

**Status:** Concluída (2026-04-10).  
**Nota:** UI inclui campos de frase/termo e botão “Gerar verso”.

1. Implementar `api/translation.rs`: montar prompts para os 3 modelos substituindo variáveis, fazer chamada HTTP, retornar texto
2. Expor como comando `generate_back`

**Verificação:** invocar os 3 modelos com frase e termo hardcoded, ver respostas coerentes no console.

---

### Fase 5 — Frontend completo

**Status:** Parcial.  
**Feito:** UI básica, captura/OCR, seleção de modelo, gerar verso, teste Anki.  
**Pendente:** tokenização clicável, modo lápis, presets de formatação, hotkey global, diálogo de configurações completo.

1. Implementar tokenização no frontend: receber array de tokens, renderizar como spans clicáveis
2. Implementar modo de edição (ícone lápis ↔ textarea)
3. Implementar seletores de modelo e preset
4. Integrar hotkey → `capture_selection` / OCR → renderização de tokens
5. Integrar botão "Gerar" → `generate_back` → campo verso
6. Integrar botão "Adicionar ao Anki" → aplicar preset → `add_note`
7. Implementar área de status e mensagens de erro
8. Implementar diálogo de configurações (deck, API key, model, hotkey)

**Verificação:** fluxo completo end-to-end funciona sem erros.

---

### Fase 6 — Polimento

**Status:** Não iniciado.

1. CSS: tema escuro/claro seguindo preferência do sistema (`prefers-color-scheme`)
2. Tratamento de todos os erros listados na Seção 4
3. Script `install.sh`: instala dependências apt, compila e instala o binário
4. README com instruções de instalação e uso
5. Ícone da janela

---

## 13. Instruções para o Agente

> **Leia esta seção antes de qualquer outra.**

### Princípios gerais

- Seguir o plano de fases na ordem. Não pular fases.
- Cada fase deve compilar e passar na verificação antes de continuar.
- Código simples e explícito. Sem abstrações prematuras.
- Nenhum `.unwrap()` em código de produção. Usar `Result` e `?` em toda lógica de negócio.
- Nenhuma dependência além das listadas na Seção 2. Se uma crate adicional parecer necessária, justificar antes de adicionar.

### Decisões já tomadas — não alterar

- **Linguagem:** Rust. Não sugerir reescrita.
- **GUI:** Tauri v2 com HTML/CSS/JS vanilla no frontend. Não usar React, Vue, Svelte ou qualquer framework JS.
- **OCR:** Tesseract local. Não usar APIs de OCR na nuvem.
- **Card template Anki:** Basic padrão. Não criar note types customizados.
- **Gestão de vocabulário:** inexistente. O programa não rastreia o que o usuário sabe.
- **Formatação do termo:** aplicada pelo programa via presets. Nunca delegar formatação ao modelo de linguagem.

### Sobre performance

A UI nunca deve travar aguardando I/O. Especificamente:

- Toda chamada de rede (API, AnkiConnect) e OCR deve rodar em `tokio::spawn` ou equivalente assíncrono.
- O hotkey listener não deve interferir com a responsividade da janela.
- Evitar clones desnecessários de strings grandes.

### Extensibilidade para japonês (futuro)

O idioma-alvo atual é inglês. A arquitetura deve permitir adicionar suporte a japonês sem reescrita:

- `source_language` e `target_language` são variáveis de configuração passadas aos prompts — nunca hardcoded no código.
- O campo `ocr_language` no config é passado diretamente ao Tesseract.
- A tokenização no frontend é por espaço e pontuação por ora. Ao adicionar japonês, será necessário tokenização morfológica no backend — deixar esse ponto de extensão explícito como comentário no código de tokenização.

### O que o agente NÃO deve fazer

- Não adicionar scoring automático de I+1 ou qualquer lógica de inferência de vocabulário.
- Não criar banco de dados local de nenhum tipo.
- Não implementar sincronização com o Anki além do `addNote`.
- Não implementar histórico de cards minerados.
- Não implementar system tray.
- Não usar frameworks CSS (Bootstrap, Tailwind) — CSS vanilla apenas.
