import {
  buildFrontPreviewHtml,
  getPresetTemplate,
  renderPlainText,
} from "./card.js";
import { hasTauri, invokeCommand, listenEvent } from "./tauri-api.js";
import { setStatus } from "./ui-state.js";
import { nextStep, stepMeta, usesTextarea } from "./wizard.js";

const elements = {
  dim: document.getElementById("dim"),
  stepIndicator: document.getElementById("step-indicator"),
  fieldLabel: document.getElementById("field-label"),
  field: document.getElementById("field"),
  area: document.getElementById("area"),
  status: document.getElementById("status"),
};

const state = {
  step: "idle",
  sentence: "",
  term: "",
  back: "",
  formatPresets: [],
  defaultDeck: "",
  defaultModel: "intermediario",
  defaultPreset: "negrito",
  noteType: "Basic",
  busy: false,
};

initializeApp();

function initializeApp() {
  document.addEventListener("keydown", handleGlobalKeydown);
  elements.field.addEventListener("keydown", handleFieldKeydown);
  elements.area.addEventListener("keydown", handleFieldKeydown);

  listenEvent("summon", () => {
    void onSummon();
  });

  if (!hasTauri()) {
    setStatus(elements.status, "Tauri API não encontrada.");
    return;
  }

  void loadDefaults();
}

async function loadDefaults() {
  try {
    const bootstrap = await invokeCommand("get_ui_bootstrap");
    state.formatPresets = bootstrap.format_presets || [];
    state.defaultDeck = bootstrap.default_deck || "";
    state.defaultModel = bootstrap.default_model || state.defaultModel;
    state.defaultPreset =
      bootstrap.default_format_preset || state.defaultPreset;
    applyTheme(bootstrap.theme || "light");
  } catch (err) {
    setStatus(elements.status, `Erro ao carregar config: ${String(err)}`);
  }

  try {
    const models = await invokeCommand("anki_get_model_names");
    if (models.length > 0) {
      state.noteType = models[0];
    }
  } catch {
    // Anki fechado: mantém "Basic", o erro aparece no envio.
  }

  try {
    const decks = await invokeCommand("anki_get_deck_names");
    if (state.defaultDeck && decks.includes(state.defaultDeck)) {
      // mantém o padrão
    } else if (decks.length > 0) {
      state.defaultDeck = decks[0];
    }
  } catch {
    // Anki fechado: mantém o padrão do config.
  }
}

function applyTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
}

async function onSummon() {
  state.step = "idle";
  state.sentence = "";
  state.term = "";
  state.back = "";
  elements.dim.hidden = false;
  showStep("sentence");
  await captureIntoSentence();
}

async function captureIntoSentence() {
  setStatus(elements.status, "Capturando seleção...");
  try {
    const text = await invokeCommand("capture_selection");
    if (!text) {
      elements.field.value = "";
      setStatus(
        elements.status,
        "Nenhuma seleção detectada. Selecione o texto e pressione Enter para tentar de novo."
      );
      elements.field.focus();
      return;
    }
    state.sentence = text;
    elements.field.value = text;
    elements.field.select();
    setStatus(elements.status, "Revise a frase e pressione Enter.");
  } catch (err) {
    setStatus(elements.status, String(err));
  }
}

function showStep(step) {
  state.step = step;
  const meta = stepMeta(step);
  elements.stepIndicator.textContent = `${meta.position}/${meta.total} · ${meta.label}`;
  elements.fieldLabel.textContent = meta.label;

  const useArea = usesTextarea(step);
  elements.field.hidden = useArea;
  elements.area.hidden = !useArea;

  if (step === "sentence") {
    elements.field.value = state.sentence;
  } else if (step === "term") {
    elements.field.value = state.term;
  } else if (step === "back") {
    elements.area.value = state.back;
  }

  activeElement().focus();
}

function activeElement() {
  return usesTextarea(state.step) ? elements.area : elements.field;
}

function handleGlobalKeydown(event) {
  if (event.key === "Escape" && !elements.dim.hidden) {
    event.preventDefault();
    void dismiss();
  }
}

function handleFieldKeydown(event) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    void advance();
  }
}

async function advance() {
  if (state.busy || elements.dim.hidden) {
    return;
  }

  if (state.step === "sentence") {
    const value = elements.field.value.trim();
    if (!value) {
      await captureIntoSentence();
      return;
    }
    state.sentence = value;
    showStep(nextStep("sentence"));
  } else if (state.step === "term") {
    const value = elements.field.value.trim();
    if (!value) {
      setStatus(elements.status, "Digite o termo desconhecido.");
      return;
    }
    state.term = value;
    await generateBack();
  } else if (state.step === "back") {
    const value = elements.area.value.trim();
    if (!value) {
      setStatus(elements.status, "O verso está vazio.");
      return;
    }
    state.back = value;
    await sendToAnki();
  }
}

async function generateBack() {
  state.busy = true;
  setStatus(elements.status, "Gerando verso...");
  try {
    const back = await invokeCommand("generate_back", {
      sentence: state.sentence,
      term: state.term,
      model: state.defaultModel,
    });
    state.back = back;
    showStep("back");
    setStatus(elements.status, "Revise o verso e pressione Enter para enviar.");
  } catch (err) {
    setStatus(elements.status, String(err));
    showStep("term");
  } finally {
    state.busy = false;
  }
}

async function sendToAnki() {
  state.busy = true;
  setStatus(elements.status, "Enviando para o Anki...");
  try {
    const presetTemplate = getPresetTemplate(
      state.formatPresets,
      state.defaultPreset
    );
    const front = buildFrontPreviewHtml(
      state.sentence,
      state.term,
      presetTemplate
    );
    const back = renderPlainText(state.back);

    if (!front) {
      setStatus(elements.status, "A frente do card está vazia.");
      return;
    }
    if (!state.defaultDeck) {
      setStatus(elements.status, "Nenhum baralho disponível (Anki aberto?).");
      return;
    }

    const noteId = await invokeCommand("anki_add_note", {
      front,
      back,
      model: state.noteType,
      deck: state.defaultDeck,
    });

    setStatus(elements.status, `✓ Nota adicionada (ID ${noteId}).`);
    window.setTimeout(() => {
      void dismiss();
    }, 900);
  } catch (err) {
    setStatus(elements.status, String(err));
  } finally {
    state.busy = false;
  }
}

async function dismiss() {
  try {
    await invokeCommand("dismiss");
  } catch {
    elements.dim.hidden = true;
  }
}
