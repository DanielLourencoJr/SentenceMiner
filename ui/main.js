import {
  buildFrontPreviewHtml,
  getPresetTemplate,
  renderPlainText,
} from "./card.js";
import { hasTauri, invokeCommand, listenEvent } from "./tauri-api.js";
import { setStatus } from "./ui-state.js";
import { nextStep, stepAction, stepMeta } from "./wizard.js";

const elements = {
  dim: document.getElementById("dim"),
  context: document.getElementById("context"),
  stepIndicator: document.getElementById("step-indicator"),
  fieldLabel: document.getElementById("field-label"),
  fieldSentence: document.getElementById("field-sentence"),
  fieldTerm: document.getElementById("field-term"),
  fieldBack: document.getElementById("field-back"),
  btnAdvance: document.getElementById("btn-advance"),
  status: document.getElementById("status"),
};

const STEP_FIELDS = {
  sentence: "fieldSentence",
  term: "fieldTerm",
  back: "fieldBack",
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
  for (const el of Object.values(fieldElements())) {
    el.addEventListener("keydown", handleFieldKeydown);
  }
  elements.btnAdvance.addEventListener("click", () => {
    void advance();
  });

  listenEvent("summon", () => {
    void onSummon();
  });

  if (!hasTauri()) {
    setStatus(elements.status, "Tauri API não encontrada.");
    return;
  }

  void loadDefaults();
}

function fieldElements() {
  return {
    fieldSentence: elements.fieldSentence,
    fieldTerm: elements.fieldTerm,
    fieldBack: elements.fieldBack,
  };
}

function activeField() {
  return fieldElements()[STEP_FIELDS[state.step]];
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
      elements.fieldSentence.value = "";
      setStatus(
        elements.status,
        "Nenhuma seleção detectada. Selecione o texto e pressione Ctrl+Enter para tentar de novo."
      );
      elements.fieldSentence.focus();
      return;
    }
    state.sentence = text;
    elements.fieldSentence.value = text;
    elements.fieldSentence.select();
    setStatus(elements.status, "Revise a frase e avance.");
  } catch (err) {
    setStatus(elements.status, String(err));
  }
}

function showStep(step) {
  state.step = step;
  const meta = stepMeta(step);
  elements.stepIndicator.textContent = `${meta.position}/${meta.total} · ${meta.label}`;
  elements.fieldLabel.textContent = meta.label;
  elements.btnAdvance.textContent = stepAction(step);

  for (const [key, el] of Object.entries(fieldElements())) {
    el.hidden = key !== STEP_FIELDS[step];
  }

  if (step === "sentence") {
    elements.fieldSentence.value = state.sentence;
  } else if (step === "term") {
    elements.fieldTerm.value = state.term;
  } else if (step === "back") {
    elements.fieldBack.value = state.back;
  }

  renderContext();
  activeField().focus();
}

// Mostra as respostas já dadas, para nenhuma etapa parecer "apagada".
function renderContext() {
  const lines = [];
  if (state.step !== "sentence" && state.sentence) {
    lines.push(["Frase", state.sentence]);
  }
  if (state.step === "back" && state.term) {
    lines.push(["Termo", state.term]);
  }

  elements.context.replaceChildren();
  elements.context.hidden = lines.length === 0;
  for (const [label, text] of lines) {
    const row = document.createElement("p");
    row.className = "context-row";
    const tag = document.createElement("span");
    tag.className = "context-tag";
    tag.textContent = label;
    row.appendChild(tag);
    row.appendChild(document.createTextNode(text));
    elements.context.appendChild(row);
  }
}

function handleGlobalKeydown(event) {
  if (event.key === "Escape" && !elements.dim.hidden) {
    event.preventDefault();
    void dismiss();
    return;
  }

  // Ctrl+Enter avança de qualquer campo, inclusive multilinha.
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
    if (!elements.dim.hidden && isFieldFocused()) {
      event.preventDefault();
      void advance();
    }
  }
}

function isFieldFocused() {
  return Object.values(fieldElements()).some((el) => el === document.activeElement);
}

function handleFieldKeydown(event) {
  // No input unilinha (termo), Enter avança — não há quebra de linha possível.
  if (event.key === "Enter" && !event.ctrlKey && !event.metaKey) {
    if (document.activeElement === elements.fieldTerm) {
      event.preventDefault();
      void advance();
    }
  }
}

async function advance() {
  if (state.busy || elements.dim.hidden) {
    return;
  }

  if (state.step === "sentence") {
    const value = elements.fieldSentence.value;
    if (!value.trim()) {
      await captureIntoSentence();
      return;
    }
    state.sentence = value;
    showStep(nextStep("sentence"));
  } else if (state.step === "term") {
    const value = elements.fieldTerm.value.trim();
    if (!value) {
      setStatus(elements.status, "Digite o termo desconhecido.");
      return;
    }
    state.term = value;
    await generateBack();
  } else if (state.step === "back") {
    const value = elements.fieldBack.value;
    if (!value.trim()) {
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
      sentence: state.sentence.trim(),
      term: state.term,
      model: state.defaultModel,
    });
    state.back = back;
    showStep("back");
    setStatus(elements.status, "Revise o verso e avance para enviar.");
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
      state.sentence.trim(),
      state.term,
      presetTemplate
    );
    const back = renderPlainText(state.back.trim());

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
