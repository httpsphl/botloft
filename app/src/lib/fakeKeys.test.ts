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

  it("closes a bot's request when its key is saved or the owner says not now", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const first = fake.keys.ask(scout.id, "GITHUB_TOKEN", "To open the PR");
    await expect(
      fake.call("keys.set", { botId: scout.id, name: "OTHER", value: "x", requestId: first.id }),
    ).rejects.toThrow();
    await fake.call("keys.set", {
      botId: scout.id,
      name: "GITHUB_TOKEN",
      value: "ghp-1",
      requestId: first.id,
    });
    expect(first.body).toMatchObject({ kind: "key_request", status: "saved" });
    const second = fake.keys.ask(scout.id, "STRIPE_KEY", "To read invoices");
    await fake.call("keys.dismiss", { requestId: second.id });
    expect(second.body).toMatchObject({ status: "dismissed" });
    await expect(fake.call("keys.dismiss", { requestId: second.id })).rejects.toThrow();
  });
});
