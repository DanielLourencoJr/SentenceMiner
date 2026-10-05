import { describe, it, expect } from "vitest";
import {
  STEPS,
  isStep,
  nextStep,
  stepIndex,
  stepMeta,
  usesTextarea,
} from "@ui/wizard.js";

describe("wizard steps", () => {
  it("advances sentence -> term -> back -> done", () => {
    expect(nextStep("sentence")).toBe("term");
    expect(nextStep("term")).toBe("back");
    expect(nextStep("back")).toBe("done");
  });

  it("restarts from unknown step", () => {
    expect(nextStep("done")).toBe("sentence");
    expect(nextStep("")).toBe("sentence");
  });

  it("reports index and validity", () => {
    expect(stepIndex("sentence")).toBe(0);
    expect(stepIndex("back")).toBe(2);
    expect(stepIndex("nope")).toBe(-1);
    expect(isStep("term")).toBe(true);
    expect(isStep("done")).toBe(false);
  });

  it("describes position and label", () => {
    expect(stepMeta("sentence")).toEqual({
      label: "Frase",
      position: 1,
      total: 3,
    });
    expect(stepMeta("back").position).toBe(3);
    expect(STEPS).toHaveLength(3);
  });

  it("uses textarea only for the back step", () => {
    expect(usesTextarea("back")).toBe(true);
    expect(usesTextarea("sentence")).toBe(false);
    expect(usesTextarea("term")).toBe(false);
  });
});
