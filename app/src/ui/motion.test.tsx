import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { moodOf } from "../features/bots/BotAvatar";
import { FakeBotloft } from "../lib/fake";
import { crewOpened, openBot, renderApp } from "../test/app";
import { SeenSince, useArrival } from "./motion";

afterEach(cleanup);

describe("the mascot's mood", () => {
  test("follows what the bot does", () => {
    const bot = (state: Parameters<typeof moodOf>[0]["state"], paused = false) =>
      moodOf({ state, paused });
    expect(bot("idle")).toBe("idle");
    expect(bot("busy")).toBe("working");
    expect(bot("launching")).toBe("working");
    expect(bot("needs_approval")).toBe("waiting");
    expect(bot("rate_limited")).toBe("tired");
    expect(bot("offline")).toBe("sleeping");
    expect(bot("idle", true)).toBe("sleeping");
    expect(moodOf({ state: "busy", paused: false }, true)).toBe("sleeping");
  });
});

describe("arrivals", () => {
  function Item({ at }: { at: number }) {
    return <p className={useArrival(at)}>{`item ${at}`}</p>;
  }

  test("only what came after the owner started looking animates", () => {
    render(
      <SeenSince.Provider value={100}>
        <Item at={50} />
        <Item at={150} />
      </SeenSince.Provider>,
    );
    expect(screen.getByText("item 50").className).toBe("");
    expect(screen.getByText("item 150").className).toBe("animate-rise");
  });

  test("a chat's history stays still and a new message rises in", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout");
    fake.setBotState(scout.id, "idle");
    fake.chat.add(scout.id, { kind: "reply", text: "From before" });
    fake.chat.turn(scout.id);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const messages = await screen.findByRole("list", { name: "Messages" });
    const before = within(messages).getByText("From before").closest("li");
    expect(before?.className).not.toContain("animate-rise");

    act(() => {
      fake.now = Date.now() + 1000;
      void fake.call("messages.send", { botId: scout.id, body: "Something new" });
    });
    const fresh = await within(messages).findByText("Something new");
    expect(fresh.closest("li")?.className).toContain("animate-rise");
  });
});

describe("switching views", () => {
  test("the new bot's pane rises in, and the window around it stays", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    for (const name of ["Scout", "Writer"]) {
      fake.setBotState(fake.addBot(ops.id, name, "Works").id, "idle");
    }
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const pane = () => screen.getByRole("main").lastElementChild as HTMLElement;
    const scout = pane();
    expect(scout.className).toContain("pane-rise");
    openBot("Writer");
    await screen.findByLabelText("Message to Writer");
    expect(pane()).not.toBe(scout);
    expect(pane().className).toContain("pane-rise");
  });
});
