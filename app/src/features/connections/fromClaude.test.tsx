import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { ClaudeMcpServer } from "../../lib/protocol.gen";
import { crewOpened, renderApp } from "../../test/app";

beforeEach(() => localStorage.clear());
afterEach(cleanup);

const server = (name: string, more: Partial<ClaudeMcpServer>): ClaudeMcpServer => ({
  name,
  folder: null,
  kind: "stdio",
  url: null,
  command: "npx",
  args: ["-y", `${name}-mcp`],
  headerNames: [],
  envNames: [],
  saved: false,
  problem: null,
  ...more,
});

/** Settings > Tools, then "Add" > "From Claude Code" (spec 25.8). */
async function openFromClaude(claudeCode: ClaudeMcpServer[] | null) {
  const fake = new FakeBotloft();
  fake.mcp.claudeCode = claudeCode;
  fake.addCrew("Ops");
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const settings = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(settings).getByRole("tab", { name: "Tools" }));
  fireEvent.click(await within(settings).findByRole("button", { name: "Add" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "From Claude Code" }));
  const dialog = screen.getByRole("dialog", { name: "Bring tools from Claude Code" });
  return { fake, settings, dialog };
}

describe("tools from Claude Code", () => {
  test("the owner picks a program, says they trust it, and it is copied", async () => {
    const { fake, settings, dialog } = await openFromClaude([
      server("search", { envNames: ["SEARCH_KEY"] }),
      server("docs", { folder: "C:/Users/ana/Work/site" }),
    ]);
    const search = await within(dialog).findByRole("checkbox", { name: /^search/ });
    expect(within(dialog).getByText("For every folder")).toBeDefined();
    expect(within(dialog).getByText("Only in C:/Users/ana/Work/site")).toBeDefined();
    expect(within(dialog).getByText(/Its keys \(SEARCH_KEY\) come along/)).toBeDefined();

    fireEvent.click(search);
    const bring = within(dialog).getByRole("button", { name: "Bring 1 tool" });
    expect(bring).toHaveProperty("disabled", true);
    fireEvent.click(within(dialog).getByLabelText("I understand and I trust it"));
    fireEvent.click(bring);

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Bring tools from Claude Code" })).toBeNull(),
    );
    expect(fake.calls).toContainEqual({
      method: "mcp.importClaude",
      params: { picks: [{ name: "search", folder: null }] },
    });
    expect(await within(settings).findByText("search")).toBeDefined();
  });

  test("one connected already, or one Botloft does not take, cannot be picked", async () => {
    const { dialog } = await openFromClaude([
      server("old", { kind: null, problem: { reason: "kind", kind: "sse" } }),
      server("signed", { kind: null, problem: { reason: "field", field: "oauth" } }),
    ]);
    const old = await within(dialog).findByRole("checkbox", { name: /^old/ });
    expect(old).toHaveProperty("disabled", true);
    expect(within(dialog).getByText(/connects by "sse"/)).toBeDefined();
    expect(within(dialog).getByRole("checkbox", { name: /^signed/ })).toHaveProperty(
      "disabled",
      true,
    );
    expect(within(dialog).getByText(/Connect it on claude.ai instead/)).toBeDefined();
  });

  test("a tool brought once shows as already here", async () => {
    const { fake } = await openFromClaude([server("search", {})]);
    await fake.call("mcp.save", {
      serverId: null,
      name: "search",
      kind: "stdio",
      url: null,
      command: "npx",
      args: [],
      headers: {},
      env: {},
      description: "",
    });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "From Claude Code" }));
    const again = screen.getByRole("dialog", { name: "Bring tools from Claude Code" });
    expect(await within(again).findByText("Already connected here.")).toBeDefined();
  });

  test("without Claude Code's settings it says so", async () => {
    const { dialog } = await openFromClaude(null);
    expect(
      await within(dialog).findByText("Claude Code's settings were not found on this computer."),
    ).toBeDefined();
  });
});
