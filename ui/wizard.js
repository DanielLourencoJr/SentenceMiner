export const STEPS = ["sentence", "term", "back"];

const LABELS = {
  sentence: "Sentence",
  term: "Unknown term",
  back: "Back",
};

const ACTIONS = {
  sentence: "Next",
  term: "Generate back",
  back: "Send to Anki",
};

export function stepIndex(step) {
  return STEPS.indexOf(step);
}

export function isStep(step) {
  return stepIndex(step) !== -1;
}

// Advance the wizard. Returns the next step, or "done" after the back.
export function nextStep(step) {
  const idx = stepIndex(step);
  if (idx === -1) {
    return STEPS[0];
  }
  if (idx === STEPS.length - 1) {
    return "done";
  }
  return STEPS[idx + 1];
}

export function stepMeta(step) {
  const idx = stepIndex(step);
  return {
    label: LABELS[step] ?? "",
    position: idx === -1 ? 0 : idx + 1,
    total: STEPS.length,
  };
}

// Sentence and back accept line breaks (textarea); only the term is single-line.
export function usesTextarea(step) {
  return step !== "term";
}

// Action button text for each step.
export function stepAction(step) {
  return ACTIONS[step] ?? "Next";
}
