import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { tokens } from "../../lib/format";
import type { ContextUsage } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

const holds = (usedTokens: number, windowTokens = 1_000_000): ContextUsage => ({
  usedTokens,
  windowTokens,
  autoCompactTokens: windowTokens - 33_000,
  compacting: false,
  updatedAt: 1,
});

/** A crew "Ops" with @scout, open on its chat. */
async function openScout(context: ContextUsage | null = holds(556_000)) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  fake.bot(scout.id).context = context;
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const meter = () => screen.getByRole("button", { name: /^Conversation space: / });
const compactions = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "bots.compact");

function open() {
  fireEvent.click(meter());
  return screen.getByRole("dialog", { name: "Conversation space" });
}

describe("conversation space", () => {
  test("a turn after a pause tells the conversation it sent again apart", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      // The turn seen in spec 19: almost all of it was the conversation again.
      fake.chat.turn(scout.id, null, {
        input: 2,
        cacheWrite: 72_636,
        reloaded: 72_000,
        cacheRead: 0,
        output: 983,
      });
    });
    const done = screen.getByText("Done in 4.2 s · 1.6k tokens · reloaded the conversation (72k)");
    expect(done.title).toBe(
      "Took 4.2 s. Read 638 new tokens and wrote 983. It also sent 72k of the conversation again, as the model no longer had it after a pause.",
    );
  });

  test("a little sent again is not worth a mention", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.turn(scout.id, null, {
        input: 2,
        cacheWrite: 1_400,
        reloaded: 300,
        cacheRead: 40_000,
        output: 100,
      });
    });
    expect(screen.getByText("Done in 4.2 s · 1.2k tokens")).toBeDefined();
  });

  test("token counts read short", () => {
    expect(tokens(850)).toBe("850");
    expect(tokens(24_340)).toBe("24.3k");
    expect(tokens(22_000)).toBe("22k");
    expect(tokens(556_000)).toBe("556k");
    expect(tokens(967_000)).toBe("967k");
    expect(tokens(1_000_000)).toBe("1M");
    expect(tokens(1_240_000)).toBe("1.2M");
  });

  test("it shows once Claude Code told it, and follows what arrives", async () => {
    const { fake, scout } = await openScout(null);
    expect(screen.queryByRole("button", { name: /^Conversation space/ })).toBeNull();

    act(() => fake.setContext(scout.id, holds(556_000)));
    expect(meter().getAttribute("aria-label")).toBe("Conversation space: 556k of 1M used (56%)");
    const menu = open();
    expect(within(menu).getByText("556k / 1M (56%)")).toBeDefined();
    expect(
      within(menu).getByText("411k left before it compacts by itself, at 967k."),
    ).toBeDefined();
    expect(within(menu).getByText(/Everything Scout read and wrote/)).toBeDefined();

    act(() => fake.setContext(scout.id, holds(968_500)));
    expect(within(menu).getByText("969k / 1M (97%)")).toBeDefined();
    expect(within(menu).getByText(/full enough to compact by itself/)).toBeDefined();

    act(() => fake.setContext(scout.id, { ...holds(24_300, 200_000), autoCompactTokens: null }));
    expect(within(menu).getByText("24.3k / 200k (12%)")).toBeDefined();
    expect(within(menu).getByText("It does not compact by itself.")).toBeDefined();
  });

  test("the owner compacts the conversation and the chat says so", async () => {
    const { fake, scout } = await openScout();
    const menu = open();
    fireEvent.click(within(menu).getByRole("button", { name: "Compact now" }));
    const waiting = await within(menu).findByRole("button", { name: "Compacting…" });
    expect((waiting as HTMLButtonElement).disabled).toBe(true);
    expect(compactions(fake).map((call) => call.params)).toEqual([{ botId: scout.id }]);
    expect(screen.queryByText(/compacts the conversation when/)).toBeNull();

    // Claude Code is done: the notice, then the new size.
    act(() => {
      fake.chat.add(scout.id, {
        kind: "notice",
        level: "info",
        code: "compacted",
        text: "The conversation was compacted: earlier messages are now a summary.",
      });
      fake.setContext(scout.id, holds(24_000));
    });
    expect(
      await screen.findByText(/what came before is now a summary, and there is room/),
    ).toBeDefined();
    expect(within(menu).getByText("24k / 1M (2%)")).toBeDefined();
    expect(within(menu).getByRole("button", { name: "Compact now" })).toBeDefined();
    // Housekeeping does not become the conversation's last line.
    expect(within(sidebar()).queryByText(/compacted/)).toBeNull();
  });

  test("a busy bot compacts when it finishes what it is doing", async () => {
    const { fake, scout } = await openScout();
    act(() => fake.setBotState(scout.id, "busy"));
    fireEvent.click(within(open()).getByRole("button", { name: "Compact now" }));
    expect(
      await screen.findByText("Scout compacts the conversation when it finishes what it's doing."),
    ).toBeDefined();
  });

  test("a bot that is not running cannot compact", async () => {
    const { fake, scout } = await openScout();
    act(() => fake.setBotState(scout.id, "offline"));
    const menu = open();
    const button = within(menu).getByRole("button", { name: "Compact now" });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    expect(within(menu).getByText("Scout is not running, so it cannot compact now.")).toBeDefined();
  });

  test("compacting by itself and a compaction that failed read in the owner's words", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.add(scout.id, {
        kind: "notice",
        level: "info",
        code: "auto_compacted",
        text: "The conversation was full, so it was compacted.",
      });
      fake.chat.add(scout.id, {
        kind: "notice",
        level: "warning",
        code: "compact_failed",
        text: "No messages to compact",
      });
    });
    expect(
      await screen.findByText(/was full, so it was compacted: what came before/),
    ).toBeDefined();
    expect(
      screen.getByText("The conversation could not be compacted: No messages to compact"),
    ).toBeDefined();
  });
});
