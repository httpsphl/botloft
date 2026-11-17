import { act, cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout at work, open on its chat. */
async function scoutAtWork() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const stopButton = () => screen.queryByRole("button", { name: "Stop Scout" });
const sendButton = () => screen.queryByRole("button", { name: "Send" });
const interrupts = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "bots.interrupt");

describe("stopping an agent", () => {
  test("stop takes the place of send while the agent works, and ends the turn", async () => {
    const { fake, scout } = await scoutAtWork();
    expect(sendButton()).toBeNull();
    const stop = stopButton();
    expect(stop?.title).toBe(
      "Stops what Scout is doing now. A message still waiting in line comes after.",
    );

    fireEvent.click(stop as HTMLElement);
    await waitFor(() => expect(interrupts(fake)).toHaveLength(1));
    expect(interrupts(fake)[0]?.params).toEqual({ botId: scout.id });
    // Not an error: the line says the owner stopped it, and what it spent.
    expect(await screen.findByText("Stopped by you after 3.1 s · 1.9k tokens")).toBeDefined();
    await waitFor(() => expect(stopButton()).toBeNull());
    expect(sendButton()).not.toBeNull();
  });

  test("with something written, send and stop are both there", async () => {
    await scoutAtWork();
    fireEvent.change(screen.getByRole("textbox", { name: "Message to Scout" }), {
      target: { value: "Actually, only the first part" },
    });
    expect(stopButton()).not.toBeNull();
    expect(sendButton()).not.toBeNull();
  });

  test("a tool waiting for the owner is refused", async () => {
    const { fake, scout } = await scoutAtWork();
    act(() => {
      fake.chat.ask(scout.id, "Bash", "rm -rf build");
      fake.setBotState(scout.id, "needs_approval");
    });
    fireEvent.click((await waitFor(() => stopButton())) as HTMLElement);
    await waitFor(() =>
      expect([...fake.chat.approvals.values()].map((approval) => approval.status)).toEqual([
        "denied",
      ]),
    );
  });

  test("an idle agent, or one whose agent cannot stop, shows no stop", async () => {
    const { fake, scout } = await scoutAtWork();
    act(() => fake.setBotState(scout.id, "idle"));
    await waitFor(() => expect(stopButton()).toBeNull());

    act(() => {
      const bot = fake.bot(scout.id);
      bot.agent = "codex";
      fake.changedBot(bot);
      fake.setBotState(scout.id, "busy");
    });
    expect(stopButton()).toBeNull();
    expect(sendButton()).not.toBeNull();
  });

  test("a turn Botloft stopped because nothing happened says who stopped it", async () => {
    const { fake, scout } = await scoutAtWork();
    act(() => {
      fake.chat.add(scout.id, {
        kind: "notice",
        level: "warning",
        code: "stuck_turn",
        text: "Nothing happened in this turn for 30 minutes, so Botloft stopped it.",
      });
      fake.chat.stopped(scout.id, true);
    });
    expect(
      await screen.findByText(
        "Nothing happened in this turn for 30 minutes, so Botloft stopped it. Write to the agent to try again.",
      ),
    ).toBeDefined();
    expect(screen.getByText("Stopped by Botloft after 3.1 s · 1.9k tokens")).toBeDefined();
    expect(screen.queryByText(/Stopped by you/)).toBeNull();
  });
});
