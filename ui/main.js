import {
  buildFrontPreviewHtml,
  getPresetTemplate,
  renderPlainText,
} from "./card.js";
import { hasTauri, invokeCommand, listenEvent } from "./tauri-api.js";
import { setStatus } from "./ui-state.js";
import { nextStep, resolveSentence, stepAction, stepMeta } from "./wizard.js";

const elements = {
  dim: document.getElementById("dim"),
  stepIndicator: document.getElementById("step-indicator"),
  fieldLabel: document.getElementById("field-label"),
  fieldSentence: document.getElementById("field-sentence"),
  fieldTerm: document.getElementById("field-term"),
  fieldBack: document.getElementById("field-back"),
  frontPreview: document.getElementById("front-preview"),
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
  // Term WE suggested last time: used to recognize our own PRIMARY
  // pollution on the next summon (see resolveSentence).
  lastSuggestion: "",
  formatPresets: [],
  defaultDeck: "",
  defaultModel: "intermediate",
  defaultPreset: "bold",
  noteType: "Basic",
  busy: false,
};

initializeApp();

function initializeApp() {
  document.addEventListener("keydown", handleGlobalKeydown);
  elements.fieldTerm.addEventListener("input", updateFrontPreview);
  elements.btnAdvance.addEventListener("click", () => {
    void advance();
  });

  listenEvent("summon", () => {
    void onSummon();
  });

  if (!hasTauri()) {
    setStatus(elements.status, "Tauri API not found.");
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
    setStatus(elements.status, `Error loading config: ${String(err)}`);
  }

  try {
    const models = await invokeCommand("anki_get_model_names");
    if (models.length > 0) {
      state.noteType = models[0];
    }
  } catch {
    // Anki closed: keep "Basic", the error surfaces at send time.
  }

  try {
    const decks = await invokeCommand("anki_get_deck_names");
    if (state.defaultDeck && decks.includes(state.defaultDeck)) {
      // keep the default
    } else if (decks.length > 0) {
      state.defaultDeck = decks[0];
    }
  } catch {
    // Anki closed: keep the config default.
  }
}

function applyTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
}

async function onSummon() {
  state.step = "idle";
  state.term = "";
  state.back = "";
  elements.dim.hidden = false;
  // Fresh Anki lists every summon: the startup ones go stale when Anki
  // was closed at boot. Silent on failure (backend errors at send time).
  void refreshAnkiLists();
  const fresh = await captureSelectionText();
  // Ignore our own PRIMARY pollution (see resolveSentence), then clear
  // the marker so a later summon judges against its own suggestion.
  state.sentence = resolveSentence(fresh, state.lastSuggestion, state.sentence);
  state.lastSuggestion = "";
  showStep("sentence");
  if (!state.sentence) {
    setStatus(
      elements.status,
      "No selection detected. Select the text and press Ctrl+Enter to retry."
    );
    elements.fieldSentence.focus();
    return;
  }
  setStatus(elements.status, "Review the sentence, then advance.");
}

async function captureSelectionText() {
  try {
    return await invokeCommand("capture_selection");
  } catch (err) {
    setStatus(elements.status, String(err));
    return "";
  }
}

async function refreshAnkiLists() {
  try {
    const models = await invokeCommand("anki_get_model_names");
    if (models.length > 0) {
      state.noteType = models[0];
    }
  } catch {
    // keep previous value
  }
  try {
    const decks = await invokeCommand("anki_get_deck_names");
    if (state.defaultDeck && decks.includes(state.defaultDeck)) {
      // keep the default
    } else if (decks.length > 0) {
      state.defaultDeck = decks[0];
    }
  } catch {
    // keep previous value
  }
}

async function captureIntoSentence() {
  setStatus(elements.status, "Capturing selection...");
  const fresh = await captureSelectionText();
  if (!fresh) {
    elements.fieldSentence.value = "";
    setStatus(
      elements.status,
      "No selection detected. Select the text and press Ctrl+Enter to retry."
    );
    elements.fieldSentence.focus();
    return;
  }
  // Same pollution rule as a fresh summon: our own suggested term
  // never counts as a new sentence.
  state.sentence = resolveSentence(fresh, state.lastSuggestion, state.sentence);
  elements.fieldSentence.value = state.sentence;
  // No auto-select: cursor goes to the end, so typing (or Enter)
  // appends instead of replacing everything.
  elements.fieldSentence.focus();
  elements.fieldSentence.setSelectionRange(
    elements.fieldSentence.value.length,
    elements.fieldSentence.value.length
  );
  setStatus(elements.status, "Review the sentence, then advance.");
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
  elements.frontPreview.hidden = step === "sentence";
  if (step !== "sentence") {
    updateFrontPreview();
  }

  activeField().focus();
}

// The styled front (user preset) IS the sentence display on steps
// 2 and 3 — updated on every keystroke of the term, no duplicated text.
function updateFrontPreview() {
  const presetTemplate = getPresetTemplate(
    state.formatPresets,
    state.defaultPreset
  );
  const term = state.step === "term" ? elements.fieldTerm.value : state.term;
  const html = buildFrontPreviewHtml(state.sentence, term, presetTemplate);
  elements.frontPreview.innerHTML =
    `<span class="front-tag">Front:</span> ${html}`;
}

function handleGlobalKeydown(event) {
  if (event.key === "Escape" && !elements.dim.hidden) {
    event.preventDefault();
    void dismiss();
    return;
  }

  // Ctrl+Enter advances from any field, including multiline ones.
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
    void suggestTerm();
  } else if (state.step === "term") {
    const value = elements.fieldTerm.value.trim();
    if (!value) {
      setStatus(elements.status, "Type the unknown term.");
      return;
    }
    state.term = value;
    await generateBack();
  } else if (state.step === "back") {
    const value = elements.fieldBack.value;
    if (!value.trim()) {
      setStatus(elements.status, "The back is empty.");
      return;
    }
    state.back = value;
    await sendToAnki();
  }
}

// Best-effort term suggestion: fills the field only if the user has
// not typed anything yet and the backend found a candidate.
async function suggestTerm() {
  try {
    const suggestion = await invokeCommand("infer_term", {
      sentence: state.sentence,
    });
    if (suggestion && !elements.fieldTerm.value && state.step === "term") {
      elements.fieldTerm.value = suggestion;
      // Selected: one keystroke replaces it with the right term.
      // Recorded: selecting it claims PRIMARY, so the next summon
      // knows to ignore it (see resolveSentence).
      state.lastSuggestion = suggestion;
      elements.fieldTerm.focus();
      elements.fieldTerm.select();
      updateFrontPreview();
      setStatus(elements.status, "Suggested term — edit if wrong.");
    }
  } catch {
    // Silent: the field simply stays manual.
  }
}

async function generateBack() {
  state.busy = true;
  setStatus(elements.status, "Generating back...");
  try {
    const back = await invokeCommand("generate_back", {
      sentence: state.sentence.trim(),
      term: state.term,
      model: state.defaultModel,
    });
    state.back = back;
    showStep("back");
    setStatus(elements.status, "Review the back, then advance to send.");
  } catch (err) {
    setStatus(elements.status, String(err));
    showStep("term");
  } finally {
    state.busy = false;
  }
}

async function sendToAnki() {
  state.busy = true;
  setStatus(elements.status, "Sending to Anki...");
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
      setStatus(elements.status, "The card front is empty.");
      return;
    }
    if (!state.defaultDeck) {
      setStatus(elements.status, "No deck available (Anki open?).");
      return;
    }

    const noteId = await invokeCommand("anki_add_note", {
      front,
      back,
      model: state.noteType,
      deck: state.defaultDeck,
    });

    setStatus(elements.status, `✓ Note added (ID ${noteId}).`);
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
