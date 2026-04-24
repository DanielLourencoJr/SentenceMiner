export function setStatus(statusEl, text) {
  statusEl.textContent = `Status: ${text}`;
}

export function setBusy(buttons, isBusy) {
  for (const button of buttons) {
    button.disabled = isBusy;
  }
}

export function setEditableHtml(element, html) {
  element.innerHTML = html;
}

export function clearPreview(frontPreviewEl, backPreviewEl) {
  setEditableHtml(frontPreviewEl, "");
  setEditableHtml(backPreviewEl, "");
}

export function populateSelect(selectEl, values) {
  selectEl.innerHTML = "";

  for (const value of values) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = value;
    selectEl.appendChild(option);
  }
}

export function populatePresetSelect(selectEl, presets) {
  selectEl.innerHTML = "";

  for (const preset of presets) {
    const option = document.createElement("option");
    option.value = preset.name;
    option.textContent = preset.name;
    selectEl.appendChild(option);
  }
}
