const sentenceEl = document.getElementById("sentence");
const termEl = document.getElementById("term");
const backEl = document.getElementById("back");
const cardModelEl = document.getElementById("card-model");
const btnText = document.getElementById("btn-capture-text");
const btnOcr = document.getElementById("btn-capture-ocr");
const statusEl = document.getElementById("status");
const modelSelect = document.getElementById("anki-model");
const deckSelect = document.getElementById("anki-deck");
const presetSelect = document.getElementById("format-preset");
const btnAddAnki = document.getElementById("btn-add-anki");

let formatPresets = [];
let defaultDeck = "";

setStatus("UI carregada.");

function setStatus(text) {
  statusEl.textContent = `Status: ${text}`;
}

function setBusy(isBusy) {
  btnText.disabled = isBusy;
  btnOcr.disabled = isBusy;
  btnGenerate.disabled = isBusy;
  btnAddAnki.disabled = isBusy;
}

async function loadUiBootstrap() {
  try {
    if (!(await ensureTauri())) return;
    const bootstrap = await window.__TAURI__.core.invoke("get_ui_bootstrap");
    formatPresets = bootstrap.format_presets || [];
    defaultDeck = bootstrap.default_deck || "";
    presetSelect.innerHTML = "";
    for (const preset of formatPresets) {
      const opt = document.createElement("option");
      opt.value = preset.name;
      opt.textContent = preset.name;
      presetSelect.appendChild(opt);
    }
    if (bootstrap.default_format_preset) {
      presetSelect.value = bootstrap.default_format_preset;
    }
    if (bootstrap.default_model) {
      cardModelEl.value = bootstrap.default_model;
    }
    setStatus("Presets carregados.");
  } catch (err) {
    setStatus(`Erro ao carregar presets: ${String(err)}`);
  }
}

async function loadModelNames() {
  try {
    if (!(await ensureTauri())) return;
    const models = await window.__TAURI__.core.invoke("anki_get_model_names");
    modelSelect.innerHTML = "";
    for (const name of models) {
      const opt = document.createElement("option");
      opt.value = name;
      opt.textContent = name;
      modelSelect.appendChild(opt);
    }
    if (models.length > 0) {
      setStatus(`Modelos carregados: ${models.length}`);
    }
  } catch (err) {
    setStatus(`Erro ao listar modelos: ${String(err)}`);
  }
}

async function loadDeckNames() {
  try {
    if (!(await ensureTauri())) return;
    const decks = await window.__TAURI__.core.invoke("anki_get_deck_names");
    deckSelect.innerHTML = "";
    for (const name of decks) {
      const opt = document.createElement("option");
      opt.value = name;
      opt.textContent = name;
      deckSelect.appendChild(opt);
    }
    if (defaultDeck && decks.includes(defaultDeck)) {
      deckSelect.value = defaultDeck;
    }
    if (decks.length > 0) {
      setStatus(`Baralhos carregados: ${decks.length}`);
    }
  } catch (err) {
    setStatus(`Erro ao listar baralhos: ${String(err)}`);
  }
}

loadUiBootstrap();
loadModelNames();
loadDeckNames();

if (window.__TAURI__ && window.__TAURI__.event) {
  window.__TAURI__.event.listen("anki_status", (event) => {
    if (event && event.payload) {
      setStatus(event.payload);
    }
  });

  window.__TAURI__.event.listen("capture_selection_started", () => {
    setBusy(true);
    sentenceEl.value = "";
    termEl.value = "";
    backEl.value = "";
    setStatus("Capturando selecao...");
  });

  window.__TAURI__.event.listen("hotkey_registered", (event) => {
    const hotkey = event && event.payload ? event.payload : "";
    if (hotkey) {
      setStatus(`Hotkey ativa: ${hotkey}`);
    }
  });

  window.__TAURI__.event.listen("hotkey_warning", (event) => {
    if (event && event.payload) {
      setStatus(String(event.payload));
    }
  });

  window.__TAURI__.event.listen("hotkey_triggered", (event) => {
    const state = event && event.payload ? event.payload : "";
    setStatus(`Hotkey acionada${state ? ` (${state})` : ""}.`);
  });

  window.__TAURI__.event.listen("capture_selection_result", (event) => {
    const payload = event ? event.payload : null;
    if (payload && payload.text) {
      sentenceEl.value = payload.text;
      setStatus("Selecao capturada.");
    } else if (payload && payload.error) {
      setStatus(payload.error);
    } else {
      setStatus("Nenhuma selecao detectada.");
    }
    setBusy(false);
  });
}

async function ensureTauri() {
  if (!window.__TAURI__ || !window.__TAURI__.core) {
    setStatus("Tauri API nao encontrada.");
    return false;
  }
  return true;
}

btnText.addEventListener("click", async () => {
  setBusy(true);
  sentenceEl.value = "";
  termEl.value = "";
  backEl.value = "";
  setStatus("Capturando selecao...");
  try {
    if (!(await ensureTauri())) return;

    const text = await window.__TAURI__.core.invoke("capture_selection");
    if (!text) {
      setStatus("Nenhuma selecao detectada.");
      return;
    }
    sentenceEl.value = text;
    setStatus("Selecao capturada.");
  } catch (err) {
    setStatus(String(err));
  } finally {
    setBusy(false);
  }
});

btnOcr.addEventListener("click", async () => {
  setBusy(true);
  sentenceEl.value = "";
  termEl.value = "";
  backEl.value = "";
  setStatus("Buscando ultimo screenshot...");
  try {
    if (!(await ensureTauri())) return;

    const ocrText = await window.__TAURI__.core.invoke(
      "capture_ocr_last_screenshot"
    );
    sentenceEl.value = ocrText;
    setStatus("OCR concluido.");
  } catch (err) {
    setStatus(String(err));
  } finally {
    setBusy(false);
  }
});

const btnGenerate = document.getElementById("btn-generate");
btnGenerate.addEventListener("click", async () => {
  try {
    setBusy(true);
    if (!(await ensureTauri())) return;
    const sentence = sentenceEl.value.trim();
    const term = termEl.value.trim();
    const model = cardModelEl.value;
    if (!sentence || !term) {
      setStatus("Preencha a frase e o termo.");
      return;
    }
    setStatus("Gerando verso...");
    const back = await window.__TAURI__.core.invoke("generate_back", {
      sentence,
      term,
      model,
    });
    backEl.value = back;
    setStatus("Verso gerado.");
  } catch (err) {
    setStatus(String(err));
  } finally {
    setBusy(false);
  }
});

function getPresetTemplate(name) {
  const preset = formatPresets.find((p) => p.name === name);
  return preset ? preset.template : "{term}";
}

function buildFront(sentence, term, presetTemplate) {
  const idx = sentence.indexOf(term);
  if (idx === -1) return null;
  const before = sentence.slice(0, idx);
  const after = sentence.slice(idx + term.length);
  const formatted = presetTemplate.replace("{term}", term);
  return `${before}${formatted}${after}`;
}

btnAddAnki.addEventListener("click", async () => {
  try {
    setBusy(true);
    if (!(await ensureTauri())) return;
    const sentence = sentenceEl.value.trim();
    const term = termEl.value.trim();
    const back = backEl.value.trim();
    const model = modelSelect.value || "Basic";
    const deck = deckSelect.value || defaultDeck;
    const presetName = presetSelect.value;
    if (!sentence || !term || !back) {
      setStatus("Preencha frase, termo e verso antes de enviar.");
      return;
    }
    if (!deck) {
      setStatus("Selecione um baralho valido.");
      return;
    }
    const template = getPresetTemplate(presetName);
    const front = buildFront(sentence, term, template);
    if (!front) {
      setStatus("Termo nao encontrado na frase.");
      return;
    }
    setStatus("Enviando para o Anki...");
    const noteId = await window.__TAURI__.core.invoke("anki_add_note", {
      front,
      back,
      model,
      deck,
    });
    setStatus(`Nota adicionada. ID: ${noteId}`);
  } catch (err) {
    setStatus(String(err));
  } finally {
    setBusy(false);
  }
});
