import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { App } from "./App";
import type { Client } from "./lib/client";
import { FakeBotloft } from "./lib/fake";
import { FakeHost } from "./lib/fakeHost";

// xterm.js needs a real browser; the terminal has its own tests.
vi.mock("./features/terminal/TerminalView", () => ({
  TerminalView: ({ botId }: { botId: string }) => <div data-testid={`terminal-${botId}`} />,
}));

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
    expect(within(sidebar()).getByRole("button", { name: /Ops/ })).toBeDefined();
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
    // The badge, and a callout that says what to do.
    expect(within(view).getAllByText("Needs approval")).toHaveLength(2);
    expect(within(view).getByRole("alert").textContent).toContain("permission prompt");
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

  test("the owner starts the daemon from the app", async () => {
    const host = new FakeHost();
    host.status = { state: "stopped", port: 45710, home: "C:\\data\\Botloft" };
    renderApp(new FakeBotloft(), host);
    fireEvent.click(await screen.findByRole("button", { name: "Start the daemon" }));
    expect(await screen.findByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();
    expect(host.starts).toBe(1);
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
    expect(fake.calls.at(-1)?.method).toBe("bots.archive");
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

  test("a missing Claude Code says why bots cannot start", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    fake.system = {
      ...fake.system,
      claudeVersion: null,
      runtimeError: "claude.exe was not found on PATH.",
    };
    renderApp(fake);
    const banner = await screen.findByText("Bots cannot start");
    expect(banner.closest("[role=alert]")?.textContent).toContain("claude.exe was not found");
  });

  test("a lost connection shows in the title bar", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    act(() => fake.setConnection({ kind: "waiting", retryAt: 0 }));
    expect(screen.getByText("Reconnecting…")).toBeDefined();
  });
  test("a bot's view shows its terminal, and details in a second tab", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout", "Finds sources");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    expect(screen.getByTestId(`terminal-${scout.id}`)).toBeDefined();
    fireEvent.click(screen.getByRole("tab", { name: "Details" }));
    expect(screen.getByText(scout.workspace)).toBeDefined();
    // The terminal stays mounted behind the other tab.
    expect(screen.getByTestId(`terminal-${scout.id}`)).toBeDefined();
  });

  test("a start that takes long points at the trust prompt", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    vi.useFakeTimers();
    try {
      act(() => fake.setBotState(scout.id, "launching", 2));
      expect(screen.queryByText("Still starting")).toBeNull();
      act(() => vi.advanceTimersByTime(4000));
      expect(screen.getByText("Still starting")).toBeDefined();
      act(() => fake.setBotState(scout.id, "idle"));
      expect(screen.queryByText("Still starting")).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  test("a bot that never started says how to start it", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    await fake.call("crews.setPaused", { crewId: crew.id, paused: true });
    const scout = fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    fireEvent.click(within(sidebar()).getByRole("button", { name: /Scout/ }));
    expect(screen.getByText(/has not started since the daemon did/).textContent).toContain(
      "Resume it to start it.",
    );
    expect(screen.queryByTestId(`terminal-${scout.id}`)).toBeNull();
  });
});
