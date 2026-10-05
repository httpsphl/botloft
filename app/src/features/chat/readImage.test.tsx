import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { ToolItem } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { readImagePath } from "./readImage";

afterEach(cleanup);

const read = (input: string, name = "Read") => ({ name, input }) as ToolItem;

describe("the image a bot reads", () => {
  test("is found in a Read of a picture, even with its input cut short", () => {
    expect(readImagePath(read(JSON.stringify({ file_path: "C:\\Work\\q10.png" })))).toBe(
      "C:\\Work\\q10.png",
    );
    expect(readImagePath(read('{"file_path":"C:\\\\Work\\\\shot.JPG","limit":20'))).toBe(
      "C:\\Work\\shot.JPG",
    );
    expect(readImagePath(read(JSON.stringify({ file_path: "C:\\Work\\notes.md" })))).toBeNull();
    expect(readImagePath(read(JSON.stringify({ file_path: "a.png" }), "Write"))).toBeNull();
  });

  test("shows on the call's line and larger when it opens", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const file = fake.files.add(scout.id, "q10.png", { text: "picture" });
    fake.setBotState(scout.id, "busy");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });

    act(() => {
      fake.chat.tool(scout.id, "Read", {
        summary: "q10.png",
        input: JSON.stringify({ file_path: file.path }),
      });
    });
    const now = within(chat).getByRole("button", { name: /Read a file/ });
    const thumb = await within(now).findByRole("img", { name: "q10.png" });
    expect(thumb.getAttribute("src")).toMatch(/^data:image\/png;base64,/);

    act(() => {
      fake.chat.turn(scout.id);
      fake.setBotState(scout.id, "idle");
    });
    const line = within(chat).getByRole("button", { name: /Read a file/ });
    fireEvent.click(line);
    expect(within(chat).getAllByRole("img", { name: "q10.png" })).toHaveLength(2);
  });

  test("an image outside the bot's folders keeps the plain line", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    fake.chat.tool(scout.id, "Read", {
      summary: "secret.png",
      status: "done",
      input: JSON.stringify({ file_path: "D:\\Elsewhere\\secret.png" }),
    });
    fake.chat.turn(scout.id);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });
    await within(chat).findByRole("button", { name: /Read a file/ });
    await act(async () => {});
    expect(within(chat).queryByRole("img", { name: "secret.png" })).toBeNull();
  });
});
