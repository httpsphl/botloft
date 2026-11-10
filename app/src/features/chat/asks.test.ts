import { describe, expect, test } from "vitest";
import { answers, asks } from "./asks";

// The same cases as the daemon's `asks.rs`: both must agree.
describe("asks", () => {
  test("a reply whose last line ends with a question asks", () => {
    expect(asks("The report is ready.\n\nI checked the figures. Should I send it to Acme?")).toBe(
      true,
    );
    expect(asks("Done.\n\n**Want me to deploy v2.1 now?**\n")).toBe(true);
    expect(asks("- Pode publicar? ")).toBe(true);
    expect(asks("¿Lo mando ya？")).toBe(true);
  });

  test("a reply that asks nothing last does not", () => {
    expect(asks("Should I? I already did it.")).toBe(false);
    expect(asks("Is it ready?\n\nYes, it is.")).toBe(false);
    expect(asks("")).toBe(false);
    expect(asks("\n  \n")).toBe(false);
  });

  test("only a thumb on a question answers it", () => {
    expect(answers("👍", true)).toBe(true);
    expect(answers("👎", true)).toBe(true);
    expect(answers("❤️", true)).toBe(false);
    expect(answers("👍", false)).toBe(false);
  });
});
