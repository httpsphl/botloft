import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "./App";
import type { Client } from "./lib/client";
import { FakeBotloft } from "./lib/fake";
import { FakeHost } from "./lib/fakeHost";

afterEach(cleanup);

function renderApp(fake = new FakeBotloft(), host = new FakeHost()) {
  render(<App host={host} connect={() => fake as Client} />);
  return { fake, host };
}

function type(label: string, value: string) {
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
}

/** The crews sidebar. */
const sidebar = () => screen.getByRole("navigation", { name: "Crews" });

async function newBot(name: string) {
  const [headerButton] = await screen.findAllByRole("button", { name: "New bot" });
  fireEvent.click(headerButton as HTMLElement);
  type("Name", name);
  type("Role", `${name} role`);
  fireEvent.click(screen.getByRole("button", { name: "Create bot" }));
}

describe("app", () => {
  test("a first run shows what is ready and opens the first crew", async () => {
    renderApp();
    expect(await screen.findByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();
    expect(screen.getByText("Version 2.1.284")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Create your first crew" }));
    type("Name", "Ops");
    fireEvent.click(screen.getByRole("button", { name: "Create crew" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Ops" })).toBeDefined();
    expect(within(sidebar()).getByRole("button", { name: "Ops" })).toBeDefined();
  });

  test("a new bot opens with its handle and follows its state", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await newBot("Scout");
    expect(await screen.findByRole("heading", { level: 1, name: "Scout" })).toBeDefined();
    expect(screen.getByText("@scout")).toBeDefined();
    const view = screen.getByRole("region", { name: "Scout" });
    expect(within(view).getByText("Starting")).toBeDefined();
    const bot = [...fake.bots.values()][0];
    act(() => fake.setBotState(bot?.id ?? "", "needs_approval"));
    // The request itself shows in the chat; the header says why it waits.
    expect(within(view).getByText("Needs approval")).toBeDefined();
    act(() => fake.setBotState(bot?.id ?? "", "auth_error"));
    expect(within(view).getByRole("alert").textContent).toContain("not signed in");
  });

  test("the daemon's validation errors stay in the form", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    await newBot("Scout");
    expect((await screen.findByRole("alert")).textContent).toBe(
      "@scout is already used in this crew",
    );
    expect(screen.getByRole("dialog", { name: "New bot" })).toBeDefined();
  });

  test("the app sets itself up on the first run, without asking", async () => {
    const host = new FakeHost();
    host.status = { state: "stopped", port: 45710, home: "C:\\data\\Botloft" };
    renderApp(new FakeBotloft(), host);
    expect(await screen.findByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();
    expect(host.installs).toEqual(["install"]);
    expect(screen.queryByText(/daemon/i)).toBeNull();
  });

  test("a failed setup explains itself in plain words", async () => {
    const host = new FakeHost();
    host.status = { state: "stopped", port: 45710, home: "C:\\data\\Botloft" };
    host.afterInstall = new Error("cannot register the scheduled task Botloft: access denied");
    renderApp(new FakeBotloft(), host);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Botloft couldn't start");
    expect(screen.getByText("Details")).toBeDefined();
    host.afterInstall = host.status = { ...host.status };
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(host.installs).toEqual(["install", "install"]);
  });

  test("archiving a bot asks first", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Archive bot" }));
    const dialog = screen.getByRole("dialog", { name: "Archive Scout?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Archive bot" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Ops" })).toBeDefined();
    expect(within(sidebar()).queryByRole("button", { name: /Scout/ })).toBeNull();
    expect(fake.calls.filter((call) => call.method !== "catalog.list").at(-1)?.method).toBe(
      "bots.archive",
    );
  });

  test("pausing a crew shows it everywhere", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    fake.setBotState(scout.id, "offline");
    renderApp(fake);
    fireEvent.click(await screen.findByRole("button", { name: "Pause crew" }));
    expect(await screen.findByRole("button", { name: "Resume crew" })).toBeDefined();
    expect(within(sidebar()).getAllByText("Paused")).toHaveLength(2);
  });

  test("a missing Claude Code says why bots cannot start and where to get it", async () => {
    const fake = new FakeBotloft();
    const host = new FakeHost();
    fake.addCrew("Ops");
    fake.system = {
      ...fake.system,
      claudeVersion: null,
      runtimeError: "claude.exe was not found on PATH.",
    };
    renderApp(fake, host);
    const banner = await screen.findByText("Bots can't start");
    const alert = banner.closest("[role=alert]") as HTMLElement;
    expect(alert.textContent).toContain("claude.exe was not found");
    fireEvent.click(within(alert).getByRole("button", { name: "How to install Claude Code" }));
    await waitFor(() => expect(host.opened).toEqual(["https://code.claude.com/docs/en/setup"]));
  });

  test("a lost connection shows in the title bar", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    act(() => fake.setConnection({ kind: "waiting", retryAt: 0 }));
    expect(screen.getByText("Reconnecting…")).toBeDefined();
  });
  test("a bot opens on its chat, with its details in a side panel", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout", "Finds sources");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    expect(await screen.findByText("Start a conversation with Scout")).toBeDefined();
    expect(screen.queryByText(scout.workspace)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Show details" }));
    const details = screen.getByRole("complementary", { name: "About Scout" });
    expect(within(details).getByText(scout.workspace)).toBeDefined();
    fireEvent.click(within(details).getByRole("button", { name: "Close details" }));
    expect(screen.queryByRole("complementary", { name: "About Scout" })).toBeNull();
  });

  test("a paused bot still takes messages, and says they wait", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    await fake.call("crews.setPaused", { crewId: crew.id, paused: true });
    fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    expect(await screen.findByText(/What you send waits until it runs again/)).toBeDefined();
    expect(screen.getByLabelText("Message to Scout")).toBeDefined();
  });
});
