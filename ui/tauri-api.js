export function hasTauri() {
  return Boolean(window.__TAURI__ && window.__TAURI__.core);
}

export async function invokeCommand(command, payload) {
  if (!hasTauri()) {
    throw new Error("Tauri API não encontrada.");
  }

  return window.__TAURI__.core.invoke(command, payload);
}

export function listenEvent(eventName, handler) {
  if (!window.__TAURI__ || !window.__TAURI__.event) {
    return null;
  }

  return window.__TAURI__.event.listen(eventName, handler);
}
