import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";

describe("the fake's agent keys", () => {
  it("lists names without values, keeps a description and forgets a deleted key", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    await fake.call("keys.set", {
      botId: scout.id,
      name: "WA_TOKEN",
      value: "wa-123",
      description: "WhatsApp",
    });
    const listed = await fake.call("keys.set", {
      botId: scout.id,
      name: "WA_TOKEN",
      value: "wa-456",
    });
    expect(listed).toEqual([
      { name: "WA_TOKEN", description: "WhatsApp", setAt: fake.now, missing: false },
    ]);
    expect(JSON.stringify(listed)).not.toContain("wa-456");
    expect(await fake.call("keys.delete", { botId: scout.id, name: "WA_TOKEN" })).toEqual([]);
  });
});
