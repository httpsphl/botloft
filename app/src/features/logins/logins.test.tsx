import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

/** "Ops" with Scout and Cody, and Settings open on Logins. */
async function setUp() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const cody = fake.addBot(ops.id, "Cody", "Writes code");
  fake.bot(cody.id).agent = "codex";
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const settings = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(settings).getByRole("tab", { name: "Logins" }));
  await within(settings).findByText(/Sign in to a site once/);
  return { fake, scout, cody, settings };
}

describe("shared site logins (spec 35.4)", () => {
  test("the owner signs in once in a window, and the login shows when it closes", async () => {
    const { fake, settings } = await setUp();
    expect(within(settings).getByText(/No login yet/)).toBeTruthy();
    fireEvent.change(within(settings).getByRole("textbox", { name: "Site address" }), {
      target: { value: "https://www.reddit.com/login" },
    });
    fireEvent.click(within(settings).getByRole("button", { name: "Open window" }));
    expect((await within(settings).findByRole("status")).textContent).toContain(
      "Sign in to reddit.com in the window that opened",
    );
    act(() => fake.logins.closeWindow(false));
    expect((await within(settings).findByRole("status")).textContent).toContain(
      "No login was found on reddit.com",
    );

    fireEvent.change(within(settings).getByRole("textbox", { name: "Site address" }), {
      target: { value: "reddit.com" },
    });
    fireEvent.click(within(settings).getByRole("button", { name: "Open window" }));
    await within(settings).findByText(/in the window that opened/);
    act(() => fake.logins.closeWindow(true));
    expect((await within(settings).findByRole("status")).textContent).toContain(
      "Signed in to reddit.com",
    );
    const list = within(settings).getByRole("list", { name: "Your logins" });
    expect(within(list).getByRole("button", { name: /reddit\.com/ }).textContent).toContain(
      "No agent yet",
    );
  });

  test("a card gives the login to agents of every engine, and removes it", async () => {
    const { fake, scout, cody, settings } = await setUp();
    act(() => fake.logins.add("reddit.com"));
    // The list is read when the page opens: come back to it.
    fireEvent.click(within(settings).getByRole("tab", { name: "Tools" }));
    fireEvent.click(within(settings).getByRole("tab", { name: "Logins" }));
    const list = await within(settings).findByRole("list", { name: "Your logins" });
    fireEvent.click(within(list).getByRole("button", { name: /reddit\.com/ }));
    const dialog = await screen.findByRole("dialog", { name: "reddit.com" });
    fireEvent.click(within(dialog).getByRole("switch", { name: "Scout can use it" }));
    await within(dialog).findByRole("switch", { name: "Scout can use it", checked: true });
    fireEvent.click(within(dialog).getByRole("switch", { name: "Cody can use it" }));
    await within(dialog).findByRole("switch", { name: "Cody can use it", checked: true });
    expect(fake.logins.list().logins[0]?.botIds).toEqual([scout.id, cody.id]);

    fireEvent.click(within(dialog).getByRole("button", { name: "Remove login" }));
    const confirm = await screen.findByRole("dialog", { name: "Remove the login to reddit.com?" });
    fireEvent.click(within(confirm).getByRole("button", { name: "Remove" }));
    await within(settings).findByText(/No login yet/);
    expect(fake.logins.list().logins).toEqual([]);
  });
});
