import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { lessonText } from "./lessonText";

afterEach(cleanup);

/** Scout's browser open on a sign-in page, in the owner's hands. */
async function inHands() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  fake.browser.open(scout.id, "https://shop.example/login?next=/orders", "Sign in");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  fireEvent.click(screen.getByRole("button", { name: /^Show browser/ }));
  await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
  act(() => {
    fake.browser.frame(scout.id, "AAAA");
  });
  fireEvent.click(screen.getByRole("button", { name: "Take control" }));
  await screen.findByText("You are in control");
  return { fake, scout };
}

/** The daemon reads what the owner did as steps. */
function teachSteps(fake: FakeBotloft, botId: string) {
  act(() => {
    fake.lesson.step(botId, { kind: "click", label: "E-mail", role: "textbox" });
    fake.lesson.step(botId, { kind: "type", label: "E-mail", role: "textbox" });
    fake.lesson.step(botId, { kind: "type", label: "Password", role: "textbox", secret: true });
    fake.lesson.step(botId, { kind: "press", label: "Enter" });
  });
}

async function finish(fake: FakeBotloft, botId: string) {
  fireEvent.click(screen.getByRole("button", { name: "Teach a task" }));
  const recording = await screen.findByRole("region", { name: "Recording the lesson" });
  expect(within(recording).getByText("1. Open https://shop.example/login")).toBeDefined();
  teachSteps(fake, botId);
  expect(
    within(recording).getByText('4. Type the password in "Password": ask me to type it'),
  ).toBeDefined();
  // The pill ends the lesson before giving anything back.
  fireEvent.click(screen.getByRole("button", { name: "Finish the lesson" }));
  const dialog = await screen.findByRole("dialog", { name: "Teach Scout a task" });
  fireEvent.change(within(dialog).getByLabelText("Name of the task"), {
    target: { value: "Check my orders" },
  });
  return dialog;
}

describe("teaching a task", () => {
  test("the words say where things were typed, never what", () => {
    const text = lessonText(
      "Check my orders",
      [
        { kind: "open", label: "https://shop.example/login", role: null, secret: false },
        { kind: "type", label: "E-mail", role: "textbox", secret: false },
        { kind: "type", label: "Password", role: "textbox", secret: true },
      ],
      en.browser.lesson,
    );
    expect(text).toBe(
      [
        'How to do "Check my orders" in the browser, as I showed you:',
        "1. Open https://shop.example/login",
        '2. Type in "E-mail"',
        '3. Type the password in "Password": ask me to type it',
        "",
        "Where I typed something, I don't show what: use what the task needs, or ask me.",
      ].join("\n"),
    );
  });

  test("a lesson records live and goes to the bot to remember, without the steps taken out", async () => {
    const { fake, scout } = await inHands();
    const dialog = await finish(fake, scout.id);
    fireEvent.click(within(dialog).getByRole("button", { name: "Remove step 2" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Send to Scout to remember" }));
    await waitFor(() =>
      expect(fake.calls.some((call) => call.method === "messages.send")).toBe(true),
    );
    const sent = fake.calls.find((call) => call.method === "messages.send")?.params as {
      body: string;
    };
    expect(sent.body).toContain('How to do "Check my orders"');
    expect(sent.body).toContain('2. Type in "E-mail"');
    expect(sent.body).not.toContain('Click "E-mail"');
    expect(sent.body).toContain('so you can do "Check my orders" when I ask.');
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    // The owner still has the browser.
    expect(screen.getByText("You are in control")).toBeDefined();
  });

  test("a lesson becomes a routine, its request already written", async () => {
    const { fake, scout } = await inHands();
    const dialog = await finish(fake, scout.id);
    fireEvent.click(within(dialog).getByRole("button", { name: "Make it a routine…" }));
    const routine = await screen.findByRole("dialog", { name: "New routine for Scout" });
    expect((within(routine).getByLabelText("Name") as HTMLInputElement).value).toBe(
      "Check my orders",
    );
    const prompt = within(routine).getByLabelText("What should Scout do?") as HTMLTextAreaElement;
    expect(prompt.value).toContain("1. Open https://shop.example/login");
    expect(prompt.value.endsWith("Do it now.")).toBe(true);
  });
});
