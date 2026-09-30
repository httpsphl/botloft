import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { Bot } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, open on its chat. */
async function openScout(setup?: (bot: Bot, fake: FakeBotloft) => void) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  // What Claude Code reported for a bot on Sonnet.
  fake.bot(scout.id).modelInUse = "claude-sonnet-5-5";
  fake.bot(scout.id).effortDefault = "medium";
  setup?.(fake.bot(scout.id), fake);
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const picker = () => screen.getByRole("button", { name: /^Effort: / });
const efforts = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "bots.setEffort").map((call) => call.params);

function open() {
  fireEvent.click(picker());
  const menu = screen.getByRole("dialog", { name: "Effort" });
  return { menu, slider: within(menu).queryByRole("slider", { name: "Effort" }) };
}

function slideTo(slider: HTMLElement | null, stop: number) {
  fireEvent.change(slider as HTMLElement, { target: { value: String(stop) } });
}

describe("effort", () => {
  test("a bot runs at its model's own level until the owner moves the slider", async () => {
    const { fake, scout } = await openScout();
    expect(picker().getAttribute("aria-label")).toBe("Effort: Medium");
    const { menu, slider } = open();
    expect((slider as HTMLInputElement).value).toBe("1");
    expect(slider?.getAttribute("aria-valuetext")).toBe("Medium (Recommended)");
    expect(within(menu).getByText("Faster")).toBeDefined();
    expect(within(menu).getByText("Smarter")).toBeDefined();
    expect(within(menu).getByText(/Recommended for Sonnet 5.5/)).toBeDefined();
    expect(within(menu).queryByRole("button", { name: "Use recommended" })).toBeNull();

    slideTo(slider, 3);
    // The words follow the thumb at once; the bot is told when it rests.
    expect(within(menu).getByText(/thinks a lot more/)).toBeDefined();
    expect(picker().getAttribute("aria-label")).toBe("Effort: Extra high");
    expect(efforts(fake)).toEqual([]);
    await waitFor(() => expect(efforts(fake)).toEqual([{ botId: scout.id, effort: "xhigh" }]));
    expect(screen.queryByText(/switches to/)).toBeNull();

    fireEvent.click(within(menu).getByRole("button", { name: "Use recommended" }));
    await screen.findByRole("button", { name: "Effort: Medium" });
    expect(efforts(fake).at(-1)).toEqual({ botId: scout.id, effort: "default" });
  });

  test("the recommended stop is the model's own level, not a level of the owner's", async () => {
    const { fake, scout } = await openScout((bot) => {
      bot.effort = "high";
    });
    expect(picker().getAttribute("aria-label")).toBe("Effort: High");
    const { slider } = open();
    slideTo(slider, 1);
    await waitFor(() => expect(efforts(fake)).toEqual([{ botId: scout.id, effort: "default" }]));
  });

  test("several stops in a row tell the bot only the last", async () => {
    const { fake, scout } = await openScout();
    const { slider } = open();
    slideTo(slider, 2);
    slideTo(slider, 3);
    slideTo(slider, 4);
    await waitFor(() => expect(efforts(fake)).toEqual([{ botId: scout.id, effort: "max" }]));
  });

  test("a stop left on the slider is kept when the menu closes", async () => {
    const { fake, scout } = await openScout();
    const { slider } = open();
    slideTo(slider, 0);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Effort" })).toBeNull();
    await waitFor(() => expect(efforts(fake)).toEqual([{ botId: scout.id, effort: "low" }]));
  });

  test("a busy bot changes effort when it finishes what it is doing", async () => {
    const { fake, scout } = await openScout();
    act(() => fake.setBotState(scout.id, "busy"));
    const { slider } = open();
    slideTo(slider, 0);
    expect(
      await screen.findByText("Scout switches to Low effort when it finishes what it's doing."),
    ).toBeDefined();
  });

  test("a model without effort levels has no slider", async () => {
    await openScout((bot) => {
      bot.modelInUse = "claude-haiku-4-5-20251001";
      bot.effortDefault = "none";
    });
    expect(picker().getAttribute("aria-label")).toBe("Effort: Not available");
    const { menu, slider } = open();
    expect(slider).toBeNull();
    expect(
      within(menu).getByText(
        "Haiku 4.5 has no effort levels: Scout always answers at the same pace.",
      ),
    ).toBeDefined();
    expect(within(menu).queryByRole("button", { name: "Use recommended" })).toBeNull();
  });

  test("nothing is marked until Claude Code said the model's level", async () => {
    const { fake, scout } = await openScout((bot) => {
      bot.effortDefault = null;
    });
    expect(picker().getAttribute("aria-label")).toBe("Effort: Recommended");
    const { menu, slider } = open();
    expect(slider?.hasAttribute("data-unset")).toBe(true);
    expect(within(menu).getByText("Scout uses the level recommended for its model")).toBeDefined();
    // With no model level to compare, the stop is the owner's own.
    slideTo(slider, 1);
    await waitFor(() => expect(efforts(fake)).toEqual([{ botId: scout.id, effort: "medium" }]));
  });
});
