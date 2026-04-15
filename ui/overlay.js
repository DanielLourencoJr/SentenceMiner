const selection = document.getElementById("selection");

let isDragging = false;
let startX = 0;
let startY = 0;
let startScreenX = 0;
let startScreenY = 0;

function setSelectionRect(x, y, w, h) {
  selection.style.left = `${x}px`;
  selection.style.top = `${y}px`;
  selection.style.width = `${w}px`;
  selection.style.height = `${h}px`;
}

window.addEventListener("mousedown", (e) => {
  isDragging = true;
  startX = e.clientX;
  startY = e.clientY;
  startScreenX = e.screenX;
  startScreenY = e.screenY;
  selection.style.display = "block";
  setSelectionRect(startX, startY, 1, 1);
});

window.addEventListener("mousemove", (e) => {
  if (!isDragging) return;
  const x = Math.min(startX, e.clientX);
  const y = Math.min(startY, e.clientY);
  const w = Math.abs(e.clientX - startX);
  const h = Math.abs(e.clientY - startY);
  setSelectionRect(x, y, w, h);
});

window.addEventListener("mouseup", async (e) => {
  if (!isDragging) return;
  isDragging = false;

  const endX = e.clientX;
  const endY = e.clientY;
  const x = Math.min(startX, endX);
  const y = Math.min(startY, endY);
  const w = Math.abs(endX - startX);
  const h = Math.abs(endY - startY);

  if (w < 5 || h < 5) {
    selection.style.display = "none";
    return;
  }

  if (!window.__TAURI__ || !window.__TAURI__.core) {
    return;
  }

  // Use screen coordinates for capture
  const screenX = Math.min(startScreenX, e.screenX);
  const screenY = Math.min(startScreenY, e.screenY);

  try {
    const text = await window.__TAURI__.core.invoke("capture_ocr_region", {
      x: Math.round(screenX),
      y: Math.round(screenY),
      w: Math.round(w),
      h: Math.round(h),
    });

    if (window.__TAURI__.event) {
      window.__TAURI__.event.emit("ocr_result", { text });
    }
  } catch (err) {
    if (window.__TAURI__.event) {
      window.__TAURI__.event.emit("ocr_result", { text: String(err) });
    }
  }
});
