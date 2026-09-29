import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, open on its chat. */
async function openScout(setup?: (fake: FakeBotloft, botId: string) => void) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  setup?.(fake, scout.id);
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const picker = () => screen.getByRole("button", { name: /^Model: / });
const modelCalls = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "bots.setModel");

function choose(name: RegExp) {
  fireEvent.click(picker());
  const menu = screen.getByRole("menu", { name: "Model" });
  fireEvent.click(within(menu).getByRole("menuitemradio", { name }));
}

describe("models", () => {
  test("a bot runs on the plan's default until the owner picks a model", async () => {
    const { fake, scout } = await openScout();
    expect(picker().getAttribute("aria-label")).toBe("Model: Default");
    fireEvent.click(picker());
    const menu = screen.getByRole("menu", { name: "Model" });
    const plan = within(menu).getByRole("menuitemradio", { name: /Plan default/ });
    expect(plan.getAttribute("aria-checked")).toBe("true");
    expect(within(menu).getByText(/use up your plan's limit faster/)).toBeDefined();
    fireEvent.click(within(menu).getByRole("menuitemradio", { name: /Haiku/ }));

    await screen.findByRole("button", { name: "Model: Haiku" });
    expect(modelCalls(fake).at(-1)?.params).toEqual({ botId: scout.id, model: "haiku" });
    expect(screen.queryByText(/switches to/)).toBeNull();
  });

  test("the plan's default is named once Claude Code reported it", async () => {
    await openScout((fake, botId) => {
      fake.bot(botId).modelInUse = "claude-opus-5-5";
    });
    expect(picker().getAttribute("aria-label")).toBe("Model: Opus 5.5");
    fireEvent.click(picker());
    expect(screen.getByText("Scout uses your plan's default model, now Opus 5.5")).toBeDefined();
  });

  test("a busy bot changes model when it finishes what it is doing", async () => {
    const { fake, scout } = await openScout();
    act(() => fake.setBotState(scout.id, "busy"));
    choose(/Sonnet/);
    expect(
      await screen.findByText("Scout switches to Sonnet when it finishes what it's doing."),
    ).toBeDefined();
  });

  test("a model the plan does not have is told in the owner's words", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.add(scout.id, {
        kind: "notice",
        level: "error",
        code: "model_unavailable",
        text: "Claude Code could not use this bot's model.",
      });
    });
    expect(await screen.findByText(/may not be on your Claude plan/)).toBeDefined();
  });

  test("a new bot can start on a chosen model", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const [headerButton] = await screen.findAllByRole("button", { name: "New bot" });
    fireEvent.click(headerButton as HTMLElement);
    const dialog = screen.getByRole("dialog", { name: "New bot" });
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Writer" } });
    fireEvent.change(within(dialog).getByLabelText("Model"), { target: { value: "sonnet" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create bot" }));
    await screen.findByRole("button", { name: "Model: Sonnet" });
    const create = fake.calls.find((call) => call.method === "bots.create");
    expect(create?.params).toMatchObject({ name: "Writer", model: "sonnet" });
  });
});
