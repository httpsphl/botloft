import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { matching, slashQuery } from "./slash";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

function crewOf() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  const cody = fake.addBot(ops.id, "Cody", "Codes");
  fake.bot(cody.id).agent = "codex";
  fake.skills.slash.set(scout.id, [
    { name: "code-review", description: "", from: "claude_code" },
    { name: "humanizer", description: "Rewrites text so it reads like a person.", from: "owner" },
    { name: "vercel:deploy", description: "Ships the site.", from: "plugin" },
  ]);
  return { fake, scout, writer };
}

async function fieldOf(name: string) {
  await crewOpened("Ops");
  openBot(name);
  return (await screen.findByRole("textbox", {
    name: `Message to ${name}`,
  })) as HTMLTextAreaElement;
}

describe("the menu of / (spec 38.11)", () => {
  test("the query is the name after a lone slash, and names that start with it come first", () => {
    expect(slashQuery("/")).toBe("");
    expect(slashQuery("/hum")).toBe("hum");
    expect(slashQuery("/humanizer fix this")).toBeNull();
    expect(slashQuery("hello /x")).toBeNull();
    const names = matching(
      [
        { name: "code-review", description: "", from: "claude_code" },
        { name: "vercel:deploy", description: "", from: "plugin" },
        { name: "copy", description: "", from: "owner" },
      ],
      "c",
    ).map((command) => command.name);
    expect(names).toEqual(["code-review", "copy", "vercel:deploy"]);
  });

  test("lists the agent's skills, filters as you type, and Enter puts one in", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const field = await fieldOf("Scout");
    fireEvent.change(field, { target: { value: "/" } });
    const list = await screen.findByRole("listbox", { name: "Skills and commands" });
    const all = within(list).getAllByRole("option");
    expect(all).toHaveLength(3);
    expect(all[1]?.textContent).toContain("/humanizer");
    expect(all[1]?.textContent).toContain("Rewrites text so it reads like a person.");
    expect(all[1]?.textContent).toContain("Yours");

    fireEvent.change(field, { target: { value: "/c" } });
    const options = within(screen.getByRole("listbox")).getAllByRole("option");
    expect(options.map((option) => option.textContent?.split(" ")[0])).toEqual([
      expect.stringContaining("/code-review"),
      expect.stringContaining("/vercel:deploy"),
    ]);
    fireEvent.keyDown(field, { key: "ArrowDown" });
    expect(
      within(screen.getByRole("listbox")).getAllByRole("option")[1]?.getAttribute("aria-selected"),
    ).toBe("true");
    fireEvent.keyDown(field, { key: "Enter" });
    expect(field.value).toBe("/vercel:deploy ");
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(fake.calls.some((call) => call.method === "messages.send")).toBe(false);
  });

  test("a click picks, Esc closes, and the menu says when nothing matches", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const field = await fieldOf("Scout");
    fireEvent.change(field, { target: { value: "/hu" } });
    fireEvent.click(await screen.findByRole("option", { name: /humanizer/ }));
    expect(field.value).toBe("/humanizer ");

    fireEvent.change(field, { target: { value: "/co" } });
    await screen.findByRole("listbox");
    fireEvent.keyDown(field, { key: "Escape" });
    expect(screen.queryByRole("listbox")).toBeNull();

    fireEvent.change(field, { target: { value: "/zzz" } });
    expect(await screen.findByText('No skill has "zzz" in its name.')).toBeDefined();
  });

  test("an agent with no skills says so, and one on another engine has no menu", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const writer = await fieldOf("Writer");
    fireEvent.change(writer, { target: { value: "/" } });
    expect(
      await screen.findByText(
        "Writer has no skills to run by name yet. Give it some in Settings > Skills.",
      ),
    ).toBeDefined();

    const cody = await fieldOf("Cody");
    fireEvent.change(cody, { target: { value: "/" } });
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(screen.queryByText(/Looking for skills/)).toBeNull();
  });
});
