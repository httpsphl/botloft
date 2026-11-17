import { describe, expect, test } from "vitest";
import { nextFidget, seeded } from "./fidgets";

describe("fidgets", () => {
  test("never the same one twice in a row", () => {
    const draw = { random: seeded("bot_scout"), last: null };
    let last = nextFidget(draw);
    for (let i = 0; i < 200; i++) {
      const next = nextFidget(draw);
      expect(next).not.toBe(last);
      last = next;
    }
  });

  test("each mascot keeps its own rhythm: the same seed, the same fidgets", () => {
    const run = (seed: string) => {
      const draw = { random: seeded(seed), last: null };
      return Array.from({ length: 12 }, () => nextFidget(draw)).join(",");
    };
    expect(run("bot_scout")).toBe(run("bot_scout"));
    expect(run("bot_scout")).not.toBe(run("bot_writer"));
  });

  test("every fidget comes up, the bob most", () => {
    const draw = { random: seeded("seed"), last: null };
    const counts: Record<string, number> = {};
    for (let i = 0; i < 3000; i++) {
      const fidget = nextFidget(draw);
      counts[fidget] = (counts[fidget] ?? 0) + 1;
    }
    expect(Object.keys(counts).sort()).toEqual(["bob", "stretch", "sway"]);
    expect(counts.bob).toBeGreaterThan(counts.stretch ?? 0);
  });
});
