import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";

describe("the fake's bot catalog", () => {
  it("lists the roles, by category too, and reads one with its instructions", async () => {
    const fake = new FakeBotloft();
    expect(await fake.call("catalog.list", {})).toHaveLength(66);
    const product = await fake.call("catalog.list", { category: "product" });
    expect(product).toHaveLength(14);
    expect(await fake.call("catalog.list", { category: "marketing" })).toHaveLength(18);
    expect(await fake.call("catalog.list", { category: "design" })).toHaveLength(9);
    const code = await fake.call("catalog.list", { category: "code" });
    expect(code.map((role) => role.id).slice(0, 3)).toEqual([
      "developer",
      "code-reviewer",
      "qa-tester",
    ]);
    expect(code).toHaveLength(11);
    expect(code[0]).not.toHaveProperty("instructions");

    const reviewer = await fake.call("catalog.get", { id: "code-reviewer" });
    expect(reviewer.effort).toBe("high");
    expect(reviewer.instructions).toContain("### What you do");
    await expect(fake.call("catalog.get", { id: "wizard" })).rejects.toThrow();
  });

  it("adds a role as a plain bot, with the name and role the app wrote", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Studio");
    const seen: string[] = [];
    fake.subscribe((event) => seen.push(event.name));

    const bot = await fake.call("catalog.add", {
      crewId: crew.id,
      templateId: "code-reviewer",
      name: "Revisor de código",
      role: "Revisa o que os outros bots mudaram",
    });
    const sheet = await fake.call("catalog.get", { id: "code-reviewer" });
    expect(bot.name).toBe("Revisor de código");
    expect(bot.instructions).toBe(sheet.instructions);
    expect(bot.effort).toBe("high");
    expect(bot.permissionMode).toBe("default");
    expect(seen).toContain("bot.changed");

    await expect(
      fake.call("catalog.add", {
        crewId: crew.id,
        templateId: "code-reviewer",
        name: "Revisor de código",
        role: "x",
      }),
    ).rejects.toThrow();
  });
});
