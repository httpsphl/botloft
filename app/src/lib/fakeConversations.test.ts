import { describe, expect, it } from "vitest";
import { mainConversationOf } from "./conversations";
import { FakeBotloft } from "./fake";

describe("the fake's conversations", () => {
  it("opens, names, archives and brings back a conversation beside the main one", async () => {
    const fake = new FakeBotloft();
    const seen: string[] = [];
    fake.subscribe((event) => seen.push(event.name));
    const bot = fake.addBot(fake.addCrew("Ops").id, "Scout");

    const [main] = await fake.call("conversations.list", { botId: bot.id });
    expect(main?.id).toBe(mainConversationOf(bot.id));
    expect(main?.main).toBe(true);

    const made = await fake.call("conversations.create", { botId: bot.id });
    expect(made.state).toBe("sleeping");
    expect(seen).toContain("conversation.changed");
    const named = await fake.call("conversations.rename", {
      conversationId: made.id,
      title: " Trip plans ",
    });
    expect(named).toMatchObject({ title: "Trip plans", titleBy: "owner" });
    await expect(
      fake.call("conversations.rename", { conversationId: main?.id ?? "", title: "Mine" }),
    ).rejects.toThrow("main conversation");

    await fake.call("conversations.archive", { conversationId: made.id });
    expect(await fake.call("conversations.list", { botId: bot.id })).toHaveLength(1);
    expect(await fake.call("conversations.archived", { botId: bot.id })).toHaveLength(1);
    const back = await fake.call("conversations.unarchive", { conversationId: made.id });
    expect(back.archivedAt).toBeNull();
    expect(await fake.call("conversations.list", { botId: bot.id })).toHaveLength(2);
  });
});
