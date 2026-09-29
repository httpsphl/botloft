import { describe, expect, test } from "vitest";
import { modelFamily, modelName } from "./models";

describe("model names", () => {
  test("Claude ids read as their family and version", () => {
    expect(modelName("claude-opus-5-5")).toBe("Opus 5.5");
    expect(modelName("claude-haiku-4-5-20251001")).toBe("Haiku 4.5");
    expect(modelName("claude-fable-5-1")).toBe("Fable 5.1");
    expect(modelName("claude-sonnet-5")).toBe("Sonnet 5");
    expect(modelFamily("claude-sonnet-5-5")).toBe("sonnet");
  });

  test("other ids stay as they are", () => {
    expect(modelName("gpt-oss")).toBe("gpt-oss");
    expect(modelFamily("claude-opus-5-5[1m]")).toBeNull();
  });
});
