import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, scout };
}

async function openScout(fake: FakeBotloft) {
  const rendered = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return rendered;
}

const chat = () => screen.getByRole("list", { name: "Messages" });
const field = () => screen.getByLabelText("Message to Scout") as HTMLTextAreaElement;

function write(text: string) {
  fireEvent.change(field(), { target: { value: text } });
}

describe("chat", () => {
  test("Enter sends, Shift+Enter does not, and the bubble follows the delivery", async () => {
    const { fake } = crew();
    await openScout(fake);
    write("Find three sources\non retrieval");
    fireEvent.keyDown(field(), { key: "Enter", shiftKey: true });
    expect(fake.calls.some((call) => call.method === "messages.send")).toBe(false);
    fireEvent.keyDown(field(), { key: "Enter" });
    const bubble = await within(chat()).findByText(/Find three sources/);
    expect(bubble.textContent).toBe("Find three sources\non retrieval");
    expect(field().value).toBe("");
    const row = bubble.closest("li") as HTMLElement;
    expect(within(row).getByText("Waiting for the bot")).toBeDefined();
    const sent = fake.conversation.messages.at(-1);
    act(() => {
      fake.conversation.read(fake.conversation.deliveryOf(sent?.id ?? "").id);
    });
    expect(within(row).getByText("Read")).toBeDefined();
    expect(within(sidebar()).getByText("You: Find three sources on retrieval")).toBeDefined();
  });

  test("the reply streams in, then tools and the turn show under the bot", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    act(() => {
      fake.setBotState(scout.id, "busy");
      fake.chat.delta(scout.id, "Looking ");
      fake.chat.delta(scout.id, "now.");
    });
    expect(within(chat()).getByText("Looking now.")).toBeDefined();
    act(() => {
      fake.chat.reply(scout.id, "Looking **now**.");
    });
    expect(within(chat()).getByText("now").tagName).toBe("STRONG");
    expect(within(chat()).queryByText("Looking now.")).toBeNull();
    let call = fake.chat.items[0];
    act(() => {
      call = fake.chat.tool(scout.id, "Bash", {
        summary: "Run the tests",
        input: '{"command":"npm test"}',
      });
    });
    const line = within(chat()).getByRole("button", { name: /Run a command/ });
    expect(within(line).getByText("Run the tests")).toBeDefined();
    expect(within(line).getByLabelText("Running")).toBeDefined();
    act(() => {
      if (call) {
        fake.chat.finish(call, "12 passed");
      }
      fake.chat.turn(scout.id);
      fake.setBotState(scout.id, "idle");
    });
    expect(within(line).getByLabelText("Done")).toBeDefined();
    fireEvent.click(line);
    expect(within(chat()).getByText("12 passed")).toBeDefined();
    expect(within(chat()).getByText(/"command": "npm test"/)).toBeDefined();
    expect(within(chat()).getByText("Done in 4.2 s")).toBeDefined();
  });

  test("an approval waits for the owner, and a denial carries a note", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    act(() => {
      fake.chat.ask(scout.id, "Bash", "rm -rf build", '{"command":"rm -rf build"}');
    });
    const card = screen.getByRole("region", { name: "Scout asks to run a command" });
    expect(within(card).getByText("rm -rf build")).toBeDefined();
    fireEvent.change(within(card).getByLabelText("Note for Scout if you deny"), {
      target: { value: "keep the cache" },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Deny" }));
    expect(await within(chat()).findByText("Denied: run a command")).toBeDefined();
    expect(fake.calls.at(-1)).toEqual({
      method: "approvals.answer",
      params: { approvalId: expect.any(String), allow: false, note: "keep the cache" },
    });

    act(() => {
      fake.chat.ask(scout.id, "Write", "notes.md");
    });
    const next = screen.getByRole("region", { name: "Scout asks to write a file" });
    fireEvent.click(within(next).getByRole("button", { name: "Allow" }));
    expect(await within(chat()).findByText("Allowed: write a file")).toBeDefined();
  });

  test("files are picked or dropped, shown before sending, and sent with the text", async () => {
    const { fake } = crew();
    await openScout(fake);
    const image = new File([new Uint8Array([137, 80, 78, 71])], "shot.png", { type: "image/png" });
    const pdf = new File(["%PDF"], "brief.pdf", { type: "application/pdf" });
    const extra = new File(["x"], "extra.txt", { type: "text/plain" });
    fireEvent.change(screen.getByTestId("file-picker"), { target: { files: [image, pdf] } });
    const chips = await screen.findByRole("list", { name: "Files to send" });
    await within(chips).findByText("brief.pdf");
    const area = screen.getByRole("region", { name: "Chat with Scout" });
    fireEvent.drop(area, { dataTransfer: { files: [extra], types: ["Files"] } });
    await within(chips).findByText("extra.txt");
    fireEvent.click(screen.getByRole("button", { name: "Remove extra.txt" }));
    write("See attached");
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    const sent = await within(chat()).findByText("See attached");
    const row = sent.closest("li") as HTMLElement;
    expect(within(row).getByRole("img", { name: "shot.png" })).toBeDefined();
    expect(within(row).getByText("brief.pdf")).toBeDefined();
    const call = fake.calls.find((entry) => entry.method === "messages.send");
    expect(call?.params).toEqual({
      botId: expect.any(String),
      body: "See attached",
      attachments: [
        { name: "shot.png", mediaType: "image/png", data: "iVBORw==" },
        { name: "brief.pdf", mediaType: "application/pdf", data: "JVBERg==" },
      ],
    });
    expect(screen.queryByRole("list", { name: "Files to send" })).toBeNull();
  });

  test("more than ten files are refused before sending", async () => {
    const { fake } = crew();
    await openScout(fake);
    const files = Array.from({ length: 11 }, (_, n) => new File(["x"], `${n}.txt`));
    fireEvent.change(screen.getByTestId("file-picker"), { target: { files } });
    expect((await screen.findByRole("alert")).textContent).toBe(
      "A message can carry up to 10 files.",
    );
  });

  test("images sent earlier are read back from the bot's folder", async () => {
    const { fake, scout } = crew();
    await fake.call("messages.send", {
      botId: scout.id,
      body: "",
      attachments: [{ name: "chart.png", mediaType: "image/png", data: "iVBORw==" }],
    });
    await openScout(fake);
    const picture = await within(chat()).findByRole("img", { name: "chart.png" });
    expect(picture.getAttribute("src")).toBe("data:image/png;base64,iVBORw==");
    expect(fake.calls.some((call) => call.method === "attachments.read")).toBe(true);
  });

  test("links open in the browser and remote images are not loaded", async () => {
    const { fake, scout } = crew();
    fake.chat.reply(
      scout.id,
      "See [the docs](https://example.com/docs) ![pixel](https://t.example/p.png)",
    );
    const { host } = await openScout(fake);
    fireEvent.click(await within(chat()).findByRole("link", { name: "the docs" }));
    await vi.waitFor(() => expect(host.opened).toContain("https://example.com/docs"));
    expect(within(chat()).queryByRole("img")).toBeNull();
    expect(within(chat()).getByRole("link", { name: "pixel" })).toBeDefined();
  });

  test("earlier messages load on demand", async () => {
    const { fake, scout } = crew();
    for (let n = 1; n <= 60; n += 1) {
      fake.chat.reply(scout.id, `note ${n}`);
    }
    await openScout(fake);
    await within(chat()).findByText("note 60");
    expect(within(chat()).queryByText("note 10")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Load earlier messages" }));
    expect(await within(chat()).findByText("note 1")).toBeDefined();
    expect(screen.queryByRole("button", { name: "Load earlier messages" })).toBeNull();
  });
});
