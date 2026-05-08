import { describe, it, expect } from "vitest";
import {
  escapeHtml,
  renderPlainText,
  getPresetTemplate,
  buildFront,
  buildFrontPreviewHtml,
} from "../card.js";

// ─── escapeHtml ───────────────────────────────────────────────────────

describe("escapeHtml", () => {
  it("returns plain text unchanged", () => {
    expect(escapeHtml("hello world")).toBe("hello world");
  });

  it("escapes ampersand", () => {
    expect(escapeHtml("a & b")).toBe("a &amp; b");
  });

  it("escapes less than", () => {
    expect(escapeHtml("a < b")).toBe("a &lt; b");
  });

  it("escapes greater than", () => {
    expect(escapeHtml("a > b")).toBe("a &gt; b");
  });

  it("escapes all special characters combined", () => {
    expect(escapeHtml("<a & b>")).toBe("&lt;a &amp; b&gt;");
  });

  it("returns empty string unchanged", () => {
    expect(escapeHtml("")).toBe("");
  });
});

// ─── renderPlainText ──────────────────────────────────────────────────

describe("renderPlainText", () => {
  it("renders text without newlines", () => {
    expect(renderPlainText("hello world")).toBe("hello world");
  });

  it("replaces single newline with br", () => {
    expect(renderPlainText("hello\nworld")).toBe("hello<br>world");
  });

  it("replaces multiple newlines", () => {
    expect(renderPlainText("a\nb\nc")).toBe("a<br>b<br>c");
  });

  it("escapes HTML in rendered text", () => {
    expect(renderPlainText("<hello>")).toBe("&lt;hello&gt;");
  });

  it("escapes and converts newlines together", () => {
    expect(renderPlainText("<a>\n<b>")).toBe("&lt;a&gt;<br>&lt;b&gt;");
  });

  it("returns empty string unchanged", () => {
    expect(renderPlainText("")).toBe("");
  });
});

// ─── getPresetTemplate ────────────────────────────────────────────────

describe("getPresetTemplate", () => {
  const presets = [
    { name: "negrito", template: "<b>{term}</b>" },
    { name: "laranja", template: '<span style="color:#e05c00">{term}</span>' },
  ];

  it("returns template for known preset name", () => {
    expect(getPresetTemplate(presets, "negrito")).toBe("<b>{term}</b>");
  });

  it("returns template for another known preset", () => {
    expect(getPresetTemplate(presets, "laranja")).toBe(
      '<span style="color:#e05c00">{term}</span>',
    );
  });

  it("returns default template for unknown name", () => {
    expect(getPresetTemplate(presets, "sublinhado")).toBe("{term}");
  });

  it("returns default template for empty presets array", () => {
    expect(getPresetTemplate([], "negrito")).toBe("{term}");
  });

  it("returns default template for case mismatch", () => {
    expect(getPresetTemplate(presets, "Negrito")).toBe("{term}");
  });
});

// ─── buildFront ───────────────────────────────────────────────────────

describe("buildFront", () => {
  const preset = "<b>{term}</b>";

  it("formats term at start of sentence", () => {
    expect(buildFront("hello world", "hello", preset)).toBe(
      "<b>hello</b> world",
    );
  });

  it("formats term in middle of sentence", () => {
    expect(buildFront("the quick brown fox", "quick", preset)).toBe(
      "the <b>quick</b> brown fox",
    );
  });

  it("formats term at end of sentence", () => {
    expect(buildFront("hello world", "world", preset)).toBe(
      "hello <b>world</b>",
    );
  });

  it("returns null when term not found", () => {
    expect(buildFront("hello world", "missing", preset)).toBeNull();
  });

  it("formats first occurrence for repeated term", () => {
    expect(buildFront("foo bar foo", "foo", preset)).toBe(
      "<b>foo</b> bar foo",
    );
  });

  it("escapes HTML in term", () => {
    expect(buildFront("a <test> b", "<test>", preset)).toBe(
      "a <b>&lt;test&gt;</b> b",
    );
  });

  it("escapes HTML in surrounding text", () => {
    expect(buildFront("a < b > c", "b", preset)).toBe(
      "a &lt; <b>b</b> &gt; c",
    );
  });

  it("handles term with special regex characters", () => {
    expect(buildFront("cost $10", "$10", preset)).toBe(
      "cost <b>$10</b>",
    );
  });

  it("handles multi-word term", () => {
    expect(buildFront("look it up", "it up", preset)).toBe(
      "look <b>it up</b>",
    );
  });

  it("handles empty sentence", () => {
    expect(buildFront("", "word", preset)).toBeNull();
  });
});

// ─── buildFrontPreviewHtml ────────────────────────────────────────────

describe("buildFrontPreviewHtml", () => {
  const preset = "<b>{term}</b>";

  it("returns empty string for empty sentence", () => {
    expect(buildFrontPreviewHtml("", "word", preset)).toBe("");
  });

  it("renders plain sentence when term is empty", () => {
    expect(buildFrontPreviewHtml("hello world", "", preset)).toBe(
      "hello world",
    );
  });

  it("returns formatted front when term is found", () => {
    expect(buildFrontPreviewHtml("hello world", "world", preset)).toBe(
      "hello <b>world</b>",
    );
  });

  it("falls back to plain sentence when term not found", () => {
    expect(buildFrontPreviewHtml("hello world", "missing", preset)).toBe(
      "hello world",
    );
  });

  it("escapes HTML when falling back to plain sentence", () => {
    expect(buildFrontPreviewHtml("a < b", "c", preset)).toBe(
      "a &lt; b",
    );
  });

  it("passes through different preset template", () => {
    const underlinePreset = "<u>{term}</u>";
    expect(buildFrontPreviewHtml("say hi", "hi", underlinePreset)).toBe(
      "say <u>hi</u>",
    );
  });
});
