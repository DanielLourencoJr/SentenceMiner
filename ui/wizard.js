export const STEPS = ["sentence", "term", "back"];

const LABELS = {
  sentence: "Frase",
  term: "Termo desconhecido",
  back: "Verso",
};

const ACTIONS = {
  sentence: "Avançar",
  term: "Gerar verso",
  back: "Enviar ao Anki",
};

export function stepIndex(step) {
  return STEPS.indexOf(step);
}

export function isStep(step) {
  return stepIndex(step) !== -1;
}

// Avança o wizard. Retorna o próximo passo, ou "done" após o verso.
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

// Frase e verso aceitam quebra de linha (textarea); só o termo é unilinha.
export function usesTextarea(step) {
  return step !== "term";
}

// Texto do botão de ação de cada passo.
export function stepAction(step) {
  return ACTIONS[step] ?? "Avançar";
}
