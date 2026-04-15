export function escapeHtml(text) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

export function renderPlainText(text) {
  return escapeHtml(text).replaceAll("\n", "<br>");
}

export function getPresetTemplate(formatPresets, name) {
  const preset = formatPresets.find((item) => item.name === name);
  return preset ? preset.template : "{term}";
}

export function buildFront(sentence, term, presetTemplate) {
  const idx = sentence.indexOf(term);
  if (idx === -1) {
    return null;
  }

  const before = renderPlainText(sentence.slice(0, idx));
  const after = renderPlainText(sentence.slice(idx + term.length));
  const formatted = presetTemplate.replace("{term}", escapeHtml(term));
  return `${before}${formatted}${after}`;
}

export function buildFrontPreviewHtml(sentence, term, presetTemplate) {
  if (!sentence) {
    return "";
  }

  if (!term) {
    return renderPlainText(sentence);
  }

  return buildFront(sentence, term, presetTemplate) ?? renderPlainText(sentence);
}
