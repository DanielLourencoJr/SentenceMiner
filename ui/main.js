import {
  buildFrontPreviewHtml,
  getPresetTemplate,
  renderPlainText,
} from "./card.js";
import { hasTauri, invokeCommand, listenEvent } from "./tauri-api.js";
import {
  clearPreview,
  populatePresetSelect,
  populateSelect,
  setBusy,
  setEditableHtml,
  setStatus,
} from "./ui-state.js";

const elements = {
  sentence: document.getElementById("sentence"),
  term: document.getElementById("term"),
  frontPreview: document.getElementById("front-preview"),
  backPreview: document.getElementById("back-preview"),
  cardModel: document.getElementById("card-model"),
  btnCaptureText: document.getElementById("btn-capture-text"),
  btnCaptureOcr: document.getElementById("btn-capture-ocr"),
  status: document.getElementById("status"),
  ankiModel: document.getElementById("anki-model"),
  ankiDeck: document.getElementById("anki-deck"),
  preset: document.getElementById("format-preset"),
  btnAddAnki: document.getElementById("btn-add-anki"),
  btnGenerate: document.getElementById("btn-generate"),
};

const state = {
  formatPresets: [],
  defaultDeck: "",
};

const busyButtons = [
  elements.btnCaptureText,
  elements.btnCaptureOcr,
  elements.btnGenerate,
  elements.btnAddAnki,
];

setStatus(elements.status, "UI carregada.");
registerDomEvents();
initializeApp();

function registerDomEvents() {
  elements.btnCaptureText.addEventListener("click", handleCaptureSelectionClick);
  elements.btnCaptureOcr.addEventListener("click", handleCaptureOcrClick);
  elements.btnGenerate.addEventListener("click", handleGenerateClick);
  elements.btnAddAnki.addEventListener("click", handleAddAnkiClick);

  elements.sentence.addEventListener("input", updateFrontPreview);
  elements.term.addEventListener("input", updateFrontPreview);
  elements.preset.addEventListener("change", updateFrontPreview);
}

async function initializeApp() {
  updateFrontPreview();
  registerTauriEvents();

  if (!ensureTauri()) {
    return;
  }

  await loadUiBootstrap();
  await loadModelNames();
  await loadDeckNames();
}

function registerTauriEvents() {
  listenEvent("anki_status", (event) => {
    if (event && event.payload) {
      setStatus(elements.status, event.payload);
    }
  });

  listenEvent("capture_selection_started", () => {
    resetCapturedContent();
    setBusy(busyButtons, true);
    setStatus(elements.status, "Capturando seleção...");
  });

  listenEvent("hotkey_registered", (event) => {
    const hotkey = event && event.payload ? event.payload : "";
    if (hotkey) {
      setStatus(elements.status, `Hotkey ativa: ${hotkey}`);
    }
  });

  listenEvent("hotkey_warning", (event) => {
    if (event && event.payload) {
      setStatus(elements.status, String(event.payload));
    }
  });

  listenEvent("hotkey_triggered", (event) => {
    const stateText = event && event.payload ? event.payload : "";
    setStatus(
      elements.status,
      `Hotkey acionada${stateText ? ` (${stateText})` : ""}.`
    );
  });

  listenEvent("capture_selection_result", (event) => {
    const payload = event ? event.payload : null;

    if (payload && payload.text) {
      applyCapturedSentence(payload.text, "Seleção capturada.");
    } else if (payload && payload.error) {
      setStatus(elements.status, payload.error);
    } else {
      setStatus(elements.status, "Nenhuma seleção detectada.");
    }

    setBusy(busyButtons, false);
  });
}

function ensureTauri() {
  if (hasTauri()) {
    return true;
  }

  setStatus(elements.status, "Tauri API não encontrada.");
  return false;
}

function resetCapturedContent() {
  elements.sentence.value = "";
  elements.term.value = "";
  clearPreview(elements.frontPreview, elements.backPreview);
}

function applyCapturedSentence(text, statusMessage) {
  elements.sentence.value = text;
  updateFrontPreview();
  setStatus(elements.status, statusMessage);
}

function updateFrontPreview() {
  const sentence = elements.sentence.value.trim();
  const term = elements.term.value.trim();
  const presetTemplate = getPresetTemplate(
    state.formatPresets,
    elements.preset.value
  );

  const html = buildFrontPreviewHtml(sentence, term, presetTemplate);
  setEditableHtml(elements.frontPreview, html);
}

async function loadUiBootstrap() {
  try {
    const bootstrap = await invokeCommand("get_ui_bootstrap");
    state.formatPresets = bootstrap.format_presets || [];
    state.defaultDeck = bootstrap.default_deck || "";

    populatePresetSelect(elements.preset, state.formatPresets);

    if (bootstrap.default_format_preset) {
      elements.preset.value = bootstrap.default_format_preset;
    }

    if (bootstrap.default_model) {
      elements.cardModel.value = bootstrap.default_model;
    }

    updateFrontPreview();
    setStatus(elements.status, "Presets carregados.");
  } catch (err) {
    setStatus(elements.status, `Erro ao carregar presets: ${String(err)}`);
  }
}

async function loadModelNames() {
  try {
    const models = await invokeCommand("anki_get_model_names");
    populateSelect(elements.ankiModel, models);

    if (models.length > 0) {
      setStatus(elements.status, `Modelos carregados: ${models.length}`);
    }
  } catch (err) {
    setStatus(elements.status, `Erro ao listar modelos: ${String(err)}`);
  }
}

async function loadDeckNames() {
  try {
    const decks = await invokeCommand("anki_get_deck_names");
    populateSelect(elements.ankiDeck, decks);

    if (state.defaultDeck && decks.includes(state.defaultDeck)) {
      elements.ankiDeck.value = state.defaultDeck;
    }

    if (decks.length > 0) {
      setStatus(elements.status, `Baralhos carregados: ${decks.length}`);
    }
  } catch (err) {
    setStatus(elements.status, `Erro ao listar baralhos: ${String(err)}`);
  }
}

async function handleCaptureSelectionClick() {
  setBusy(busyButtons, true);
  resetCapturedContent();
  setStatus(elements.status, "Capturando seleção...");

  try {
    if (!ensureTauri()) {
      return;
    }

    const text = await invokeCommand("capture_selection");
    if (!text) {
      setStatus(elements.status, "Nenhuma seleção detectada.");
      return;
    }

    applyCapturedSentence(text, "Seleção capturada.");
  } catch (err) {
    setStatus(elements.status, String(err));
  } finally {
    setBusy(busyButtons, false);
  }
}

async function handleCaptureOcrClick() {
  setBusy(busyButtons, true);
  resetCapturedContent();
  setStatus(elements.status, "Buscando último screenshot...");

  try {
    if (!ensureTauri()) {
      return;
    }

    const ocrText = await invokeCommand("capture_ocr_last_screenshot");
    applyCapturedSentence(ocrText, "OCR concluído.");
  } catch (err) {
    setStatus(elements.status, String(err));
  } finally {
    setBusy(busyButtons, false);
  }
}

async function handleGenerateClick() {
  try {
    setBusy(busyButtons, true);
    if (!ensureTauri()) {
      return;
    }

    const sentence = elements.sentence.value.trim();
    const term = elements.term.value.trim();
    const model = elements.cardModel.value;

    if (!sentence || !term) {
      setStatus(elements.status, "Preencha a frase e o termo.");
      return;
    }

    setStatus(elements.status, "Gerando verso...");
    const back = await invokeCommand("generate_back", { sentence, term, model });
    setEditableHtml(elements.backPreview, renderPlainText(back));
    setStatus(elements.status, "Verso gerado.");
  } catch (err) {
    setStatus(elements.status, String(err));
  } finally {
    setBusy(busyButtons, false);
  }
}

async function handleAddAnkiClick() {
  try {
    setBusy(busyButtons, true);
    if (!ensureTauri()) {
      return;
    }

    const sentence = elements.sentence.value.trim();
    const term = elements.term.value.trim();
    const front = elements.frontPreview.innerHTML.trim();
    const back = elements.backPreview.innerHTML.trim();
    const model = elements.ankiModel.value || "Basic";
    const deck = elements.ankiDeck.value || state.defaultDeck;

    if (!sentence || !term || !back) {
      setStatus(elements.status, "Preencha frase, termo e verso antes de enviar.");
      return;
    }

    if (!deck) {
      setStatus(elements.status, "Selecione um baralho válido.");
      return;
    }

    if (!front) {
      setStatus(elements.status, "A frente do card está vazia.");
      return;
    }

    setStatus(elements.status, "Enviando para o Anki...");
    const noteId = await invokeCommand("anki_add_note", {
      front,
      back,
      model,
      deck,
    });

    setStatus(elements.status, `Nota adicionada. ID: ${noteId}`);
  } catch (err) {
    setStatus(elements.status, String(err));
  } finally {
    setBusy(busyButtons, false);
  }
}
