import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { pastedKeys } from "./pasted";

afterEach(cleanup);

const TOKEN = "ghp-live-1234567890";

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const saved = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "keys.set")?.params as
    | { name: string; value: string; requestId?: string }
    | undefined;

describe("an agent asking for a key", () => {
  test("the owner types it in a hidden field and the card shrinks to its name", async () => {
    const { fake, scout } = await openScout();
    let id = "";
    act(() => {
      id = fake.keys.ask(scout.id, "GITHUB_TOKEN", "To open the pull request").id;
    });
    const card = screen.getByRole("region", { name: "Scout needs a key" });
    expect(within(card).getByText("To open the pull request")).toBeDefined();
    const field = within(card).getByLabelText("Key GITHUB_TOKEN");
    expect(field.getAttribute("type")).toBe("password");
    fireEvent.click(within(card).getByRole("button", { name: "Show the key" }));
    expect(field.getAttribute("type")).toBe("text");
    fireEvent.change(field, { target: { value: TOKEN } });
    fireEvent.click(within(card).getByRole("button", { name: "Save" }));
    expect(await screen.findByText("You saved the key GITHUB_TOKEN")).toBeDefined();
    expect(saved(fake)).toMatchObject({ name: "GITHUB_TOKEN", value: TOKEN, requestId: id });
    expect(screen.queryByText(TOKEN)).toBeNull();
  });

  test("not now closes the request without a key", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.keys.ask(scout.id, "STRIPE_KEY", "To read invoices");
    });
    const card = screen.getByRole("region", { name: "Scout needs a key" });
    fireEvent.click(within(card).getByRole("button", { name: "Not now" }));
    expect(await screen.findByText("You did not give the key STRIPE_KEY")).toBeDefined();
    expect(fake.keys.saved.size).toBe(0);
  });
});

describe("the Keys part of an agent's details", () => {
  test("adds, marks a missing value, replaces and deletes a key", async () => {
    const { fake, scout } = await openScout();
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const details = screen.getByRole("complementary", { name: "About Scout" });
    expect(await within(details).findByText(/^No keys yet/)).toBeDefined();

    fireEvent.click(within(details).getByRole("button", { name: "Add a key" }));
    fireEvent.change(within(details).getByLabelText("Name"), {
      target: { value: "wa token" },
    });
    expect((within(details).getByLabelText("Name") as HTMLInputElement).value).toBe("WA_TOKEN");
    fireEvent.change(within(details).getByLabelText("Key"), { target: { value: TOKEN } });
    fireEvent.change(within(details).getByLabelText("Description (optional)"), {
      target: { value: "WhatsApp gateway" },
    });
    fireEvent.click(within(details).getByRole("button", { name: "Save" }));
    const list = await within(details).findByRole("list", { name: "Keys" });
    expect(within(list).getByText("WA_TOKEN")).toBeDefined();
    expect(within(list).getByText(/^WhatsApp gateway · changed/)).toBeDefined();
    expect(within(list).queryByText("Value missing")).toBeNull();

    const kept = fake.keys.saved.get(scout.id)?.get("WA_TOKEN");
    if (kept) {
      kept.missing = true;
    }
    fireEvent.click(within(list).getByRole("button", { name: "Replace WA_TOKEN" }));
    fireEvent.change(within(details).getByLabelText("Key WA_TOKEN"), {
      target: { value: "ghp-live-0987654321" },
    });
    fireEvent.click(within(details).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(fake.keys.saved.get(scout.id)?.get("WA_TOKEN")?.value).toBe("ghp-live-0987654321"),
    );

    fireEvent.click(within(details).getByRole("button", { name: "Delete WA_TOKEN" }));
    const confirm = screen.getByRole("dialog", { name: "Delete the key WA_TOKEN?" });
    fireEvent.click(within(confirm).getByRole("button", { name: "Delete" }));
    expect(await within(details).findByText(/^No keys yet/)).toBeDefined();
  });

  test("a key without its value on this computer says so", async () => {
    const { fake, scout } = await openScout();
    await fake.call("keys.set", { botId: scout.id, name: "WA_TOKEN", value: TOKEN });
    const kept = fake.keys.saved.get(scout.id)?.get("WA_TOKEN");
    if (kept) {
      kept.missing = true;
    }
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const details = screen.getByRole("complementary", { name: "About Scout" });
    expect(await within(details).findByText("Value missing")).toBeDefined();
  });
});

describe("a saved key pasted in the chat", () => {
  test("the agent gets the name and the owner is told", async () => {
    const { fake, scout } = await openScout();
    await fake.call("keys.set", { botId: scout.id, name: "WA_TOKEN", value: TOKEN });
    fireEvent.change(screen.getByRole("textbox", { name: /^Message to Scout/ }), {
      target: { value: `use ${TOKEN} please` },
    });
    fireEvent.click(screen.getByRole("button", { name: /^Send/ }));
    expect(
      await screen.findByText(
        "The key WA_TOKEN is already kept, so Scout got the name in place of the value.",
      ),
    ).toBeDefined();
    expect(screen.queryByText(new RegExp(TOKEN))).toBeNull();
  });

  test("only names the daemon swapped count, each once", () => {
    expect(pastedKeys("hi", "hi")).toEqual([]);
    expect(pastedKeys("a b a", "[chave A_KEY] b [chave A_KEY]")).toEqual(["A_KEY"]);
    expect(pastedKeys("[chave OLD] x", "[chave OLD] [chave NEW]")).toEqual(["NEW"]);
  });
});
