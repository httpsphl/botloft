import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { GUARD_MS } from "./Cards";
import { FakeIslandHost } from "./fakeHost";
import { IslandApp, PANEL_WIDTH, PILL } from "./IslandApp";

let now = 1_000_000;

beforeEach(() => {
  now = 1_000_000;
  vi.spyOn(Date, "now").mockImplementation(() => now);
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function fakeHost() {
  const host = new FakeIslandHost();
  return { host, calls: host.calls };
}

function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  return { fake, scout, writer };
}

async function openPanel() {
  fireEvent.click(await screen.findByRole("button", { name: /waiting/ }));
  return screen.findByRole("region", { name: "Botloft at the top of the screen" });
}

describe("the island", () => {
  test("stays hidden while nothing works or waits", async () => {
    const { fake } = crew();
    const { host, calls } = fakeHost();
    render(<IslandApp api={fake} host={host} />);
    await waitFor(() => expect(calls).toContain("hide"));
    expect(screen.queryByRole("region")).toBeNull();
  });

  test("is a pill that counts who waits and who works, and grows into the panel", async () => {
    const { fake, scout, writer } = crew();
    fake.setBotState(writer.id, "busy");
    fake.chat.ask(scout.id, "Bash", "git status", JSON.stringify({ command: "git status" }));
    fake.questions.ask(scout.id, "Coffee?", ["Yes", "No"]);
    const { host, calls } = fakeHost();
    render(<IslandApp api={fake} host={host} />);

    const pill = await screen.findByRole("button", { name: "2 waiting, 1 working" });
    await waitFor(() => expect(calls).toContain(`show ${PILL.width}x${PILL.height} primary`));
    fireEvent.click(pill);
    await screen.findByText("Run a command");
    await waitFor(() =>
      expect(calls.some((call) => call.startsWith(`show ${PANEL_WIDTH}x`))).toBe(true),
    );
    expect(screen.getByText("1 of 2")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Hide" }));
    expect(await screen.findByRole("button", { name: "2 waiting, 1 working" })).toBeDefined();
  });

  test("allows only after the card has been on screen a moment", async () => {
    const { fake, scout } = crew();
    fake.chat.ask(
      scout.id,
      "Bash",
      "git status",
      JSON.stringify({ command: "git status" }),
      "Shows what changed",
    );
    const { host } = fakeHost();
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();
    expect(within(island).getByText("Shows what changed")).toBeDefined();
    expect(within(island).getByText("git status")).toBeDefined();

    // Just appeared: a click that was meant for something else does nothing.
    fireEvent.click(within(island).getByRole("button", { name: "Allow" }));
    expect(fake.chat.items.find((item) => item.body.kind === "approval")?.body).toMatchObject({
      status: "pending",
    });
    now += GUARD_MS;
    fireEvent.click(within(island).getByRole("button", { name: "Allow" }));
    await waitFor(() =>
      expect(fake.chat.items.find((item) => item.body.kind === "approval")?.body).toMatchObject({
        status: "allowed",
      }),
    );
    // Opened by the owner, the panel stays; the answered request leaves it.
    await waitFor(() => expect(within(island).queryByText("Run a command")).toBeNull());
  });

  test("only denies what it cannot show whole or what belongs to Botloft", async () => {
    const { fake, scout } = crew();
    fake.chat.ask(scout.id, "Bash", "echo", '{"command": "echo xxxx…');
    const { host, calls } = fakeHost();
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();
    expect(within(island).getByText(/Too long to show whole here/)).toBeDefined();
    expect(within(island).queryByRole("button", { name: "Allow" })).toBeNull();
    expect(within(island).getByRole("button", { name: "Deny" })).toBeDefined();

    act(() => {
      fake.chat.ask(scout.id, "mcp__github__create_issue", "Fix it", "{}");
    });
    fireEvent.click(await within(island).findByRole("button", { name: "Next" }));
    expect(await within(island).findByText("Needs you in Botloft")).toBeDefined();
    expect(within(island).queryByRole("button", { name: "Allow" })).toBeNull();

    fireEvent.click(within(island).getByRole("button", { name: "Open in Botloft" }));
    await waitFor(() => expect(calls).toContain(`open ${scout.id}`));
  });

  test("answers a question with an option or its own words", async () => {
    const { fake, scout } = crew();
    const question = fake.questions.ask(scout.id, "Coffee?", ["Yes", "No"]);
    const { host } = fakeHost();
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();
    expect(within(island).getByText("Scout asks")).toBeDefined();
    now += GUARD_MS;
    fireEvent.click(within(island).getByRole("button", { name: "Yes" }));
    await waitFor(() =>
      expect(
        fake.questions.asked.find((entry) => entry.question.id === question.id)?.question,
      ).toMatchObject({ status: "answered", answer: "Yes" }),
    );
  });
});
