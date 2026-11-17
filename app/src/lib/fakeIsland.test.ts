import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";

describe("the fake's island methods", () => {
  it("lists what waits with the phone's rules, oldest first", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.chat.ask(scout.id, "Bash", "git status", JSON.stringify({ command: "git status" }));
    fake.chat.ask(scout.id, "mcp__github__create_issue", "Fix it", "{}");
    fake.chat.ask(scout.id, "Bash", "echo", `{"command": "echo ${"x".repeat(20)}…`);
    fake.questions.ask(scout.id, "Coffee?", ["Yes", "No"]);

    const cards = await fake.call("cards.list");
    expect(
      cards.approvals.map((card) => [card.text.slice(0, 10), card.cut, card.atComputer]),
    ).toEqual([
      ["git status", false, false],
      ["{}", false, true],
      ['{"command"', true, false],
    ]);
    expect(cards.approvals[0]?.bot.name).toBe("Scout");
    expect(cards.approvals[0]?.crew).toBe("Ops");
    expect(cards.questions.map((card) => card.text)).toEqual(["Coffee?"]);
  });

  it("lists the open requests for a key apart, until saved or dismissed", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const github = fake.keys.ask(scout.id, "GITHUB_TOKEN", "To open the pull request");
    fake.now += 1000;
    const stripe = fake.keys.ask(scout.id, "STRIPE_KEY", "To read the invoices");

    const cards = await fake.call("cards.list");
    expect(cards.keys).toEqual([
      {
        requestId: github.id,
        bot: { id: scout.id, name: "Scout", color: scout.color },
        crew: "Ops",
        createdAt: github.createdAt,
        name: "GITHUB_TOKEN",
        why: "To open the pull request",
      },
      expect.objectContaining({ requestId: stripe.id, name: "STRIPE_KEY" }),
    ]);
    expect(cards.approvals).toEqual([]);
    expect(cards.questions).toEqual([]);

    await fake.call("keys.set", {
      botId: scout.id,
      name: "GITHUB_TOKEN",
      value: "test-value-1",
      requestId: github.id,
    });
    await fake.call("keys.dismiss", { requestId: stripe.id });
    expect((await fake.call("cards.list")).keys).toEqual([]);
  });

  it("counts the edits and the time in turns since the start of the day", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.chat.tool(scout.id, "Edit", {
      status: "done",
      file: "C:\\work\\a.md",
      lines: { added: 3, removed: 1 },
    });
    fake.chat.turn(scout.id);

    const today = await fake.call("activity.today", { since: 0 });
    expect(today.filesEdited).toBe(1);
    expect(today.lines).toEqual({ added: 3, removed: 1 });
    expect(today.busiest?.name).toBe("Scout");
    const later = await fake.call("activity.today", { since: fake.now + 1 });
    expect(later.filesEdited).toBe(0);
    expect(later.busiest).toBeNull();
  });
});
