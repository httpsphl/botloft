import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

beforeEach(() => localStorage.clear());
afterEach(cleanup);

const LINKEDIN = JSON.stringify({
  mcpServers: {
    linkedin: { command: "uvx", args: ["linkedin-mcp-server"], env: { LI_COOKIE: "secret" } },
  },
});

async function openTools() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("dialog", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Connected tools" }));
  return { fake, scout, dialog };
}

const type = (field: HTMLElement, value: string) => fireEvent.change(field, { target: { value } });

describe("connected tools", () => {
  test("a pasted program is connected only after the owner says they trust it", async () => {
    const { fake, dialog } = await openTools();
    expect(await within(dialog).findByText("No tools connected yet.")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Connect a tool…" }));

    const adding = screen.getByRole("dialog", { name: "Connect a tool" });
    type(within(adding).getByLabelText("Paste its settings"), LINKEDIN);
    expect(within(adding).getByText("Found 1 tool.")).toBeDefined();
    expect(
      within(adding).getByText(/will run on your computer with your permissions/),
    ).toBeDefined();
    const connect = within(adding).getByRole("button", { name: "Connect" });
    expect(connect).toHaveProperty("disabled", true);
    fireEvent.click(within(adding).getByLabelText("I understand and I trust it"));
    type(within(adding).getByLabelText(/^What is it for/), "Reads people on LinkedIn.");
    fireEvent.click(connect);

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Connect a tool" })).toBeNull(),
    );
    expect(within(dialog).getByText("LinkedIn".toLowerCase())).toBeDefined();
    expect(within(dialog).getByText("No bot uses it yet")).toBeDefined();
    const saved = fake.calls.find((each) => each.method === "mcp.save");
    expect(saved?.params).toMatchObject({
      name: "linkedin",
      kind: "stdio",
      command: "uvx",
      env: { LI_COOKIE: "secret" },
      description: "Reads people on LinkedIn.",
    });
  });

  test("what is not understood is named, and nothing is saved", async () => {
    const { fake, dialog } = await openTools();
    fireEvent.click(await within(dialog).findByRole("button", { name: "Connect a tool…" }));
    const adding = screen.getByRole("dialog", { name: "Connect a tool" });
    type(
      within(adding).getByLabelText("Paste its settings"),
      JSON.stringify({ mcpServers: { a: { command: "x", timeout: 5 } } }),
    );
    expect(within(adding).getByRole("alert").textContent).toContain('"timeout"');
    expect(within(adding).getByRole("button", { name: "Connect" })).toHaveProperty(
      "disabled",
      true,
    );
    expect(fake.calls.some((each) => each.method === "mcp.save")).toBe(false);
  });

  test("settings without a name ask for one", async () => {
    const { dialog } = await openTools();
    fireEvent.click(await within(dialog).findByRole("button", { name: "Connect a tool…" }));
    const adding = screen.getByRole("dialog", { name: "Connect a tool" });
    type(
      within(adding).getByLabelText("Paste its settings"),
      JSON.stringify({ type: "http", url: "http://127.0.0.1:8000/mcp" }),
    );
    expect(within(adding).getByRole("alert").textContent).toContain("write one below");
    type(within(adding).getByLabelText("Name"), "Web");
    expect(within(adding).getByText(/leaves this computer/)).toBeDefined();
    // An address needs no "I understand": nothing runs here.
    expect(within(adding).getByRole("button", { name: "Connect" })).toHaveProperty(
      "disabled",
      false,
    );
  });

  test("a bot gets a tool when the owner turns it on, and shows how it came up", async () => {
    const { fake, scout, dialog } = await openTools();
    fireEvent.click(await within(dialog).findByRole("button", { name: "Connect a tool…" }));
    const adding = screen.getByRole("dialog", { name: "Connect a tool" });
    type(within(adding).getByLabelText("Paste its settings"), LINKEDIN);
    fireEvent.click(within(adding).getByLabelText("I understand and I trust it"));
    fireEvent.click(within(adding).getByRole("button", { name: "Connect" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Connect a tool" })).toBeNull(),
    );
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));

    openBot("Scout");
    await screen.findByRole("region", { name: "Chat with Scout" });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const details = await screen.findByRole("complementary", { name: "About Scout" });
    const toggle = await within(details).findByRole("switch", { name: "Scout can use linkedin" });
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    fireEvent.click(toggle);
    await waitFor(() => expect(toggle.getAttribute("aria-checked")).toBe("true"));
    expect(fake.calls.find((each) => each.method === "bot.mcp.set")?.params).toMatchObject({
      botId: scout.id,
    });

    // The bot started and the tool did not come up.
    const server = fake.mcp.overview().servers[0];
    act(() =>
      fake.emit({
        name: "bot.mcp",
        params: {
          botId: scout.id,
          serverIds: [server?.id ?? ("" as never)],
          states: [
            { serverId: server?.id ?? ("" as never), state: "failed", error: "Connection closed" },
          ],
        },
      }),
    );
    expect(await within(details).findByText("Did not connect")).toBeDefined();
    expect(within(details).getByText("Connection closed")).toBeDefined();
  });
});
