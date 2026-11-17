import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

const EXCEL = "C:\\Office\\EXCEL.EXE";
const NOTEPAD = "C:\\Windows\\notepad.exe";

function crewOf() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const docs = fake.addCrew("Docs");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  const editor = fake.addBot(docs.id, "Editor", "Edits");
  return { fake, scout, writer, editor };
}

/** Settings, on the Permissions page. */
async function openPermissions() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Permissions" }));
  return page;
}

describe("the Permissions page", () => {
  test("shows every agent's permissions by kind, and takes one back", async () => {
    const { fake, scout, writer } = crewOf();
    fake.desktop.grant(scout.id, EXCEL, "Excel", "act");
    fake.sites.allow(scout.id, "wikipedia.org");
    fake.allow.remember(writer.id, { toolName: "WebSearch", kind: "tool", value: "" });
    fake.crewAccess.add(scout.id, "Docs");
    renderApp(fake);
    const page = await openPermissions();

    // The desktop first, by agent, with its options.
    const scouts = await within(page).findByRole("region", { name: "Scout" });
    expect(within(scouts).getByText("Excel")).toBeDefined();
    expect(within(scouts).getByText("Can see and use")).toBeDefined();
    expect(within(page).getByRole("tab", { name: "Desktop (1)" })).toBeDefined();
    expect(within(page).getByRole("tab", { name: "Commands (1)" })).toBeDefined();
    expect(within(page).getByRole("tab", { name: "Other crews (1)" })).toBeDefined();

    // A site, taken back: the agent asks for it again.
    fireEvent.click(within(page).getByRole("tab", { name: "Sites (1)" }));
    expect(await within(page).findByText("wikipedia.org")).toBeDefined();
    fireEvent.click(within(page).getByRole("button", { name: "Take back wikipedia.org" }));
    expect(await within(page).findByText(/No site is allowed for good/)).toBeDefined();
    expect(fake.sites.sites).toEqual([]);
    expect(within(page).getByRole("tab", { name: "Sites" })).toBeDefined();

    fireEvent.click(within(page).getByRole("tab", { name: "Commands (1)" }));
    const writers = await within(page).findByRole("region", { name: "Writer" });
    expect(within(writers).getByText("Search the web")).toBeDefined();
    fireEvent.click(within(page).getByRole("tab", { name: "Other crews (1)" }));
    expect(await within(page).findByText("The whole crew Docs")).toBeDefined();
  }, 20_000);

  test("shows one agent only when chosen", async () => {
    const { fake, scout, writer } = crewOf();
    fake.desktop.grant(scout.id, EXCEL, "Excel");
    fake.desktop.grant(writer.id, NOTEPAD, "Notepad");
    renderApp(fake);
    const page = await openPermissions();
    expect(await within(page).findByRole("tab", { name: "Desktop (2)" })).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: /All agents/ }));
    fireEvent.click(screen.getByRole("option", { name: "Writer · Ops" }));
    expect(within(page).getByRole("tab", { name: "Desktop (1)" })).toBeDefined();
    expect(within(page).getByText("Notepad")).toBeDefined();
    expect(within(page).queryByText("Excel")).toBeNull();
  }, 20_000);

  test("gives an open app before the agent asks, and the whole desktop only after the risks", async () => {
    const { fake, scout } = crewOf();
    fake.desktop.apps = [
      { path: EXCEL, name: "Microsoft Excel" },
      { path: NOTEPAD, name: "Notepad" },
    ];
    renderApp(fake);
    const page = await openPermissions();
    expect(await within(page).findByText(/No agent can see apps/)).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: "Give an app" }));
    let dialog = screen.getByRole("dialog", { name: "Give an agent an app" });
    fireEvent.click(await within(dialog).findByRole("radio", { name: /Notepad/ }));
    fireEvent.click(within(dialog).getByRole("radio", { name: "Can see and use" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Give it" }));
    await waitFor(() => expect(fake.desktop.grants[0]?.appName).toBe("Notepad"));
    // The first agent listed: crews by name, Docs before Ops.
    expect(fake.desktop.grants[0]?.level).toBe("act");
    expect(await within(page).findByText("Notepad")).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: "Give an app" }));
    dialog = screen.getByRole("dialog", { name: "Give an agent an app" });
    fireEvent.click(within(dialog).getByRole("button", { name: /Editor · Docs/ }));
    fireEvent.click(screen.getByRole("option", { name: "Scout · Ops" }));
    fireEvent.click(await within(dialog).findByRole("radio", { name: "The whole desktop" }));
    const confirm = within(dialog).getByRole("button", { name: "Give it" }) as HTMLButtonElement;
    expect(confirm.disabled).toBe(true);
    expect(within(dialog).getByText(/covered in black/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("checkbox", { name: "I understand the risks" }));
    fireEvent.click(confirm);
    await waitFor(() =>
      expect(fake.desktop.grants.some((grant) => grant.scope === "desktop")).toBe(true),
    );
    expect(fake.desktop.grants.find((grant) => grant.scope === "desktop")?.botId).toBe(scout.id);
  }, 20_000);

  test("stops every agent on the desktop at once, and says what is never given", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const page = await openPermissions();
    fireEvent.click(await within(page).findByRole("button", { name: "Stop all now" }));
    await waitFor(() =>
      expect(fake.calls.some((call) => call.method === "desktop.stopAll")).toBe(true),
    );
    fireEvent.click(within(page).getByText("What is never given"));
    expect(within(page).getByText(/Password managers and vaults/)).toBeDefined();
  });

  test("opens from an agent's details on that agent, and is found by the search", async () => {
    const { fake, scout } = crewOf();
    fake.setBotState(scout.id, "idle");
    fake.sites.allow(scout.id, "example.com");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const about = screen.getByRole("complementary", { name: "About Scout" });
    expect(await within(about).findByText("example.com")).toBeDefined();
    fireEvent.click(within(about).getByRole("button", { name: "Every agent's permissions" }));

    const page = await screen.findByRole("region", { name: "Settings" });
    expect(within(page).getByRole("heading", { name: "Permissions" })).toBeDefined();
    expect(within(page).getByRole("button", { name: /Scout · Ops/ })).toBeDefined();

    const search = within(page).getByRole("searchbox");
    fireEvent.change(search, { target: { value: "real mouse" } });
    expect(within(page).getByRole("button", { name: /Real mouse and keyboard/ })).toBeDefined();
  }, 20_000);

  test("speaks the owner's language", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    await openPermissions();
    act(() => setLocaleChoice("pt-BR"));
    expect(await screen.findByRole("tab", { name: "Outras equipes" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Parar todos agora" })).toBeDefined();
  });
});

describe("only this time", () => {
  async function openScout() {
    const { fake, scout } = crewOf();
    fake.setBotState(scout.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    return { fake, scout };
  }

  test("allows an app until the agent ends its turn, and says so", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(
        scout.id,
        "mcp__botloft__desktop",
        "Notepad",
        JSON.stringify({ app: "Notepad", path: NOTEPAD, level: "see", why: "To read the list" }),
      );
    });
    const card = screen.getByRole("region", { name: "Scout asks to see Notepad" });
    fireEvent.click(within(card).getByRole("button", { name: "Only this time" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual(
        expect.objectContaining({
          method: "approvals.answer",
          params: expect.objectContaining({ allow: true, once: true }),
        }),
      ),
    );
    expect(await screen.findByText("Scout can see Notepad, only this time")).toBeDefined();
    expect(fake.desktop.grants).toEqual([]);
  });

  test("allows a site until the agent ends its turn, without keeping it", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(
        scout.id,
        "mcp__botloft__browser",
        "wikipedia.org",
        JSON.stringify({ site: "wikipedia.org", url: "https://wikipedia.org/" }),
      );
    });
    const card = screen.getByRole("region", { name: "Scout asks to use wikipedia.org" });
    fireEvent.click(within(card).getByRole("button", { name: "Only this time" }));
    expect(await screen.findByText("You allowed wikipedia.org, only this time")).toBeDefined();
    expect(fake.sites.sites).toEqual([]);
  });
});
