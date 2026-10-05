import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { CREW_ACCESS_TOOL } from "./CrewAccessCard";

afterEach(cleanup);

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

const answered = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params as
    | { allow: boolean; input?: string; note?: string }
    | undefined;

const request = (bot: string | null) =>
  JSON.stringify({
    crew: "Blog",
    bot,
    handle: bot?.toLowerCase() ?? null,
    access: ["talk", "read"],
    why: "To get this week's post.",
  });

describe("a bot asking to reach another crew", () => {
  test("for one bot: only now, always that bot, always the crew or deny", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(scout.id, CREW_ACCESS_TOOL, "Blog", request("Writer"));
    });
    const card = screen.getByRole("region", {
      name: "Scout wants to reach Writer, of the crew Blog",
    });
    expect(within(card).getByText(/^Talk with Writer/)).toBeDefined();
    expect(within(card).getByText("Read the files of Writer and of its work folder")).toBeDefined();
    expect(within(card).getByText("To get this week's post.")).toBeDefined();
    expect(within(card).getByRole("button", { name: "Only now" })).toBeDefined();
    expect(within(card).getByRole("button", { name: "Always the crew Blog" })).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Always Writer" }));
    await waitFor(() => expect(answered(fake)?.allow).toBe(true));
    expect(JSON.parse(answered(fake)?.input as string)).toEqual({ scope: "bot" });
  });

  test("for the whole crew there is no bot to always allow", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(scout.id, CREW_ACCESS_TOOL, "Blog", request(null));
    });
    const card = screen.getByRole("region", { name: "Scout wants to reach the crew Blog" });
    expect(within(card).queryByRole("button", { name: /^Always Writer/ })).toBeNull();
    fireEvent.change(within(card).getByRole("textbox"), { target: { value: "Not this week" } });
    fireEvent.click(within(card).getByRole("button", { name: "Deny" }));
    await waitFor(() =>
      expect(answered(fake)).toMatchObject({ allow: false, note: "Not this week" }),
    );
  });

  test("lasting access shows in the bot's details and can be taken back", async () => {
    const { fake, scout } = await openScout();
    fake.crewAccess.add(scout.id, "Blog");
    fake.crewAccess.add(scout.id, "Shop", { id: "bot_clerk", name: "Clerk" }, { edit: true });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const details = screen.getByRole("complementary", { name: "About Scout" });
    expect(await within(details).findByText("The whole crew Blog")).toBeDefined();
    expect(within(details).getByText("Clerk, of the crew Shop")).toBeDefined();
    expect(within(details).getByText("talk")).toBeDefined();
    expect(within(details).getByText("edit files")).toBeDefined();
    fireEvent.click(
      within(details).getByRole("button", { name: "Take back: The whole crew Blog" }),
    );
    await waitFor(() => expect(within(details).queryByText("The whole crew Blog")).toBeNull());
    expect(within(details).getByText("Clerk, of the crew Shop")).toBeDefined();
  });
});
