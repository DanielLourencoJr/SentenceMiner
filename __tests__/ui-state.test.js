import { describe, it, expect, beforeEach } from "vitest";
import {
  setStatus,
  setBusy,
  setEditableHtml,
  clearPreview,
  populateSelect,
  populatePresetSelect,
} from "@ui/ui-state.js";

// ─── setStatus ────────────────────────────────────────────────────────

describe("setStatus", () => {
  it("sets status text content with prefix", () => {
    const el = document.createElement("div");
    setStatus(el, "carregado.");
    expect(el.textContent).toBe("Status: carregado.");
  });

  it("handles empty text", () => {
    const el = document.createElement("div");
    setStatus(el, "");
    expect(el.textContent).toBe("Status: ");
  });
});

// ─── setBusy ──────────────────────────────────────────────────────────

describe("setBusy", () => {
  it("disables all buttons when busy", () => {
    const btn1 = document.createElement("button");
    const btn2 = document.createElement("button");
    setBusy([btn1, btn2], true);
    expect(btn1.disabled).toBe(true);
    expect(btn2.disabled).toBe(true);
  });

  it("enables all buttons when not busy", () => {
    const btn1 = document.createElement("button");
    const btn2 = document.createElement("button");
    btn1.disabled = true;
    btn2.disabled = true;
    setBusy([btn1, btn2], false);
    expect(btn1.disabled).toBe(false);
    expect(btn2.disabled).toBe(false);
  });

  it("handles empty array", () => {
    expect(() => setBusy([], true)).not.toThrow();
  });

  it("handles a single button", () => {
    const btn = document.createElement("button");
    setBusy([btn], true);
    expect(btn.disabled).toBe(true);
  });
});

// ─── setEditableHtml ──────────────────────────────────────────────────

describe("setEditableHtml", () => {
  it("sets innerHTML of element", () => {
    const el = document.createElement("div");
    setEditableHtml(el, "<b>hello</b>");
    expect(el.innerHTML).toBe("<b>hello</b>");
  });

  it("clears content with empty string", () => {
    const el = document.createElement("div");
    el.innerHTML = "<b>hello</b>";
    setEditableHtml(el, "");
    expect(el.innerHTML).toBe("");
  });
});

// ─── clearPreview ─────────────────────────────────────────────────────

describe("clearPreview", () => {
  it("clears both front and back elements", () => {
    const front = document.createElement("div");
    const back = document.createElement("div");
    front.innerHTML = "front text";
    back.innerHTML = "back text";

    clearPreview(front, back);

    expect(front.innerHTML).toBe("");
    expect(back.innerHTML).toBe("");
  });
});

// ─── populateSelect ───────────────────────────────────────────────────

describe("populateSelect", () => {
  it("populates from array of strings", () => {
    const select = document.createElement("select");
    populateSelect(select, ["a", "b", "c"]);
    expect(select.options.length).toBe(3);
    expect(select.options[0].value).toBe("a");
    expect(select.options[0].textContent).toBe("a");
    expect(select.options[1].value).toBe("b");
    expect(select.options[2].value).toBe("c");
  });

  it("clears existing options before populating", () => {
    const select = document.createElement("select");
    select.innerHTML = '<option value="old">old</option>';
    populateSelect(select, ["new"]);
    expect(select.options.length).toBe(1);
    expect(select.options[0].value).toBe("new");
  });

  it("handles empty array", () => {
    const select = document.createElement("select");
    select.innerHTML = '<option value="old">old</option>';
    populateSelect(select, []);
    expect(select.options.length).toBe(0);
  });
});

// ─── populatePresetSelect ─────────────────────────────────────────────

describe("populatePresetSelect", () => {
  const presets = [
    { name: "negrito", template: "<b>{term}</b>" },
    { name: "laranja", template: '<span style="color:#e05c00">{term}</span>' },
  ];

  it("populates from array of preset objects", () => {
    const select = document.createElement("select");
    populatePresetSelect(select, presets);
    expect(select.options.length).toBe(2);
    expect(select.options[0].value).toBe("negrito");
    expect(select.options[0].textContent).toBe("negrito");
    expect(select.options[1].value).toBe("laranja");
  });

  it("clears existing options", () => {
    const select = document.createElement("select");
    select.innerHTML = '<option value="old">old</option>';
    populatePresetSelect(select, presets);
    expect(select.options.length).toBe(2);
  });

  it("handles empty array", () => {
    const select = document.createElement("select");
    populatePresetSelect(select, []);
    expect(select.options.length).toBe(0);
  });
});
