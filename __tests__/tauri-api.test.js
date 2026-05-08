import { describe, it, expect, vi, afterEach } from "vitest";
import { hasTauri, invokeCommand, listenEvent } from "@ui/tauri-api.js";

function setTauriCore(mockImpl) {
  window.__TAURI__ = {
    core: {
      invoke: mockImpl,
    },
    event: {
      listen: mockImpl,
    },
  };
}

function clearTauri() {
  delete window.__TAURI__;
}

function setTauriPartial(config) {
  window.__TAURI__ = config;
}

// ─── hasTauri ─────────────────────────────────────────────────────────

describe("hasTauri", () => {
  afterEach(() => clearTauri());

  it("returns false when window.__TAURI__ is undefined", () => {
    expect(hasTauri()).toBe(false);
  });

  it("returns false when window.__TAURI__.core is undefined", () => {
    setTauriPartial({});
    expect(hasTauri()).toBe(false);
  });

  it("returns true when window.__TAURI__.core exists", () => {
    setTauriPartial({ core: {} });
    expect(hasTauri()).toBe(true);
  });
});

// ─── invokeCommand ────────────────────────────────────────────────────

describe("invokeCommand", () => {
  afterEach(() => clearTauri());

  it("throws when Tauri is not available", async () => {
    await expect(invokeCommand("test")).rejects.toThrow(
      "Tauri API não encontrada.",
    );
  });

  it("throws when passing payload without Tauri", async () => {
    await expect(invokeCommand("cmd", { key: "val" })).rejects.toThrow(
      "Tauri API não encontrada.",
    );
  });

  it("calls window.__TAURI__.core.invoke with command name", async () => {
    const invoke = vi.fn().mockResolvedValue("result");
    setTauriCore(invoke);

    const result = await invokeCommand("capture_selection");

    expect(invoke).toHaveBeenCalledWith("capture_selection", undefined);
    expect(result).toBe("result");
  });

  it("passes payload to invoke", async () => {
    const invoke = vi.fn().mockResolvedValue("ok");
    setTauriCore(invoke);

    const payload = { sentence: "hello", term: "world" };
    await invokeCommand("generate_back", payload);

    expect(invoke).toHaveBeenCalledWith("generate_back", payload);
  });

  it("returns the promise value from invoke", async () => {
    const invoke = vi.fn().mockResolvedValue(42);
    setTauriCore(invoke);

    const result = await invokeCommand("anki_add_note");

    expect(result).toBe(42);
  });

  it("propagates errors from invoke", async () => {
    const invoke = vi.fn().mockRejectedValue(new Error("Anki error"));
    setTauriCore(invoke);

    await expect(invokeCommand("anki_add_note")).rejects.toThrow("Anki error");
  });
});

// ─── listenEvent ──────────────────────────────────────────────────────

describe("listenEvent", () => {
  afterEach(() => clearTauri());

  it("returns null when window.__TAURI__ is undefined", () => {
    expect(listenEvent("event", () => {})).toBeNull();
  });

  it("returns null when window.__TAURI__.event is undefined", () => {
    setTauriPartial({ core: {} });
    expect(listenEvent("event", () => {})).toBeNull();
  });

  it("calls listen with event name and handler", () => {
    const listen = vi.fn().mockReturnValue("unlisten_fn");
    setTauriPartial({
      core: { invoke: vi.fn() },
      event: { listen },
    });

    const handler = () => {};
    const result = listenEvent("anki_status", handler);

    expect(listen).toHaveBeenCalledWith("anki_status", handler);
    expect(result).toBe("unlisten_fn");
  });

  it("returns the unlisten function from listen", () => {
    const unlisten = () => {};
    const listen = vi.fn().mockReturnValue(unlisten);
    setTauriPartial({
      core: { invoke: vi.fn() },
      event: { listen },
    });

    const result = listenEvent("test", () => {});

    expect(result).toBe(unlisten);
  });
});
