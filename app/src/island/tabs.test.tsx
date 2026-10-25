import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import type { IslandHost } from "./host";
import { IslandApp } from "./IslandApp";

afterEach(cleanup);

const host: IslandHost = {
  daemon: async () => ({ port: 0, token: "" }),
  show: async () => {},
  hide: async () => {},
  openApp: async () => {},
};

function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(writer.id, "busy");
  return { fake, scout, writer };
}

const sent = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "messages.send").map((call) => call.params);

async function openTab(name: string) {
  fireEvent.click(await screen.findByRole("button", { name: /working/ }));
  const island = await screen.findByRole("region", { name: "Botloft at the top of the screen" });
  fireEvent.click(within(island).getByRole("tab", { name }));
  return island;
}

describe("the island's Chat", () => {
  test("talks to the agent at work first, and to another one picked", async () => {
    const { fake, writer } = crew();
    fake.chat.reply(writer.id, "Starting the draft.");
    render(<IslandApp api={fake} host={host} />);
    const island = await openTab("Chat");
    expect(await within(island).findByText("Starting the draft.")).toBeDefined();

    fireEvent.change(within(island).getByRole("textbox", { name: "Message to Writer" }), {
      target: { value: "Keep it short" },
    });
    fireEvent.click(within(island).getByRole("button", { name: "Send" }));
    await waitFor(() => expect(sent(fake)).toEqual([{ botId: writer.id, body: "Keep it short" }]));

    fireEvent.click(within(island).getByRole("button", { name: "Pick an agent" }));
    fireEvent.click(within(island).getByRole("option", { name: "Scout" }));
    expect(within(island).getByRole("textbox", { name: "Message to Scout" })).toBeDefined();
  });

  test("a file dropped on the island goes with the next message", async () => {
    const { fake, writer } = crew();
    render(<IslandApp api={fake} host={host} />);
    const pill = await screen.findByRole("region", { name: "Botloft at the top of the screen" });
    const file = new File(["hello"], "notes.txt", { type: "text/plain" });
    fireEvent.drop(pill, { dataTransfer: { files: [file], types: ["Files"] } });

    const island = await screen.findByRole("region", { name: "Botloft at the top of the screen" });
    expect(await within(island).findByText("notes.txt")).toBeDefined();
    fireEvent.click(within(island).getByRole("button", { name: "Send" }));
    await waitFor(() =>
      expect(sent(fake)).toEqual([
        {
          botId: writer.id,
          body: "",
          attachments: [{ name: "notes.txt", mediaType: "text/plain", data: btoa("hello") }],
        },
      ]),
    );
    await waitFor(() => expect(within(island).queryByText("notes.txt")).toBeNull());
  });
});

describe("the island's Usage", () => {
  test("shows the plan, warns from 80% with a ring on the pill, and runs a routine now", async () => {
    const { fake, scout } = crew();
    const now = Date.now();
    fake.system = {
      ...fake.system,
      usage: {
        status: "allowed_warning",
        resetsAt: now + 3600_000,
        observedAt: now,
        windows: [
          { name: "five_hour", utilization: 0.84, resetsAt: now + 3600_000 },
          { name: "seven_day", utilization: 0.4, resetsAt: now + 86_400_000 },
        ],
      },
    };
    await fake.call("routines.create", {
      botId: scout.id,
      name: "New papers",
      prompt: "Look for new papers",
      schedule: { kind: "interval", minutes: 30 },
      timezone: "UTC",
    });
    render(<IslandApp api={fake} host={host} />);
    expect(await screen.findByTestId("plan-ring")).toBeDefined();
    expect(screen.getByRole("button", { name: "1 working, plan at 84%" })).toBeDefined();

    const island = await openTab("Usage");
    expect(await within(island).findByText(/84% used/)).toBeDefined();
    expect(within(island).getByText(/40% used/)).toBeDefined();
    expect(island.querySelectorAll('[data-warn="true"]')).toHaveLength(1);

    expect(within(island).getByText("New papers")).toBeDefined();
    fireEvent.click(within(island).getByRole("button", { name: "Run now" }));
    await waitFor(() =>
      expect(fake.calls.some((call) => call.method === "routines.runNow")).toBe(true),
    );
    expect(await within(island).findByRole("button", { name: "Started" })).toBeDefined();
  });
});
