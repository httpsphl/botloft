// Photos and files from the phone (spec 28.14): picking them, what is about
// to go, the message as it goes, and what came of it.

import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import type { ChatLine } from "../lib/protocol.gen";
import { FakePhone } from "./fake";
import { PhoneApp } from "./PhoneApp";

afterEach(cleanup);

const line: ChatLine = {
  botId: "bot_1",
  name: "Scout",
  color: "#aabbcc",
  crew: "Ops",
  state: "idle",
  lastReplyAt: 100,
  last: { kind: "reply", text: "All done", tool: null, at: 100 },
};

async function chat() {
  const phone = new FakePhone();
  phone.receive({ t: "chats", bots: [line], first: true });
  render(<PhoneApp api={phone} />);
  fireEvent.click(screen.getByRole("tab", { name: /Chats/ }));
  fireEvent.click(screen.getByRole("button", { name: /Scout/ }));
  await waitFor(() => expect(phone.asked.some((ask) => ask.t === "history")).toBe(true));
  const ask = phone.asked.find((one) => one.t === "history");
  const req = ask?.t === "history" ? ask.req : 0;
  act(() =>
    phone.receive({ t: "history", req, botId: "bot_1", items: [], more: false, done: true }),
  );
  return phone;
}

const pdf = (name = "receipt.pdf", bytes = 1200) =>
  new File([new Uint8Array(bytes)], name, { type: "application/pdf" });
const pickFiles = (files: File[]) =>
  fireEvent.change(screen.getByTestId("file-picker"), { target: { files } });
const sendButton = () => screen.getByRole("button", { name: "Send" }) as HTMLButtonElement;
const lastSend = (phone: FakePhone) => phone.asked.filter((ask) => ask.t === "send").at(-1);

describe("a file from the phone", () => {
  test("shows what is about to go, goes up sealed, and the key goes with the message", async () => {
    const phone = await chat();
    expect(screen.getByRole("button", { name: "Send a photo" })).toBeTruthy();
    pickFiles([pdf()]);
    const chosen = await screen.findByRole("list", { name: "To send" });
    expect(within(chosen).getByText("receipt.pdf")).toBeTruthy();
    // A file alone is a message too.
    expect(sendButton().disabled).toBe(false);
    fireEvent.click(sendButton());

    await waitFor(() => expect(lastSend(phone)).toBeTruthy());
    expect(phone.uploaded.map((file) => file.name)).toEqual(["receipt.pdf"]);
    expect(lastSend(phone)).toMatchObject({
      t: "send",
      botId: "bot_1",
      text: "",
      files: [{ id: "fil_fake1", name: "receipt.pdf", mediaType: "application/pdf", size: 1200 }],
    });
    expect(screen.queryByRole("list", { name: "To send" })).toBeNull();
    expect(screen.getByText("Sending…")).toBeTruthy();

    const sent = lastSend(phone);
    const clientId = sent?.t === "send" ? sent.clientId : "";
    act(() => phone.receive({ t: "sent", clientId, ok: true }));
    act(() =>
      phone.receive({
        t: "item",
        botId: "bot_1",
        item: { kind: "you", id: "itm_1", at: 5, text: "", cut: false, files: ["receipt-2.pdf"] },
      }),
    );
    // The computer kept it under another name: still one message, with its file.
    expect(screen.queryByText(/Sent\./)).toBeNull();
    expect(screen.getByRole("list", { name: "1 file" })).toBeTruthy();
    expect(screen.getByText("receipt-2.pdf")).toBeTruthy();
  });

  test("can be taken out before it goes, and goes with the words", async () => {
    const phone = await chat();
    pickFiles([pdf("a.pdf"), pdf("b.pdf")]);
    await screen.findByText("b.pdf");
    fireEvent.click(screen.getByRole("button", { name: "Remove a.pdf" }));
    expect(screen.queryByText("a.pdf")).toBeNull();
    fireEvent.change(screen.getByLabelText("Message to Scout"), { target: { value: "For May" } });
    fireEvent.click(sendButton());
    await waitFor(() => expect(lastSend(phone)).toBeTruthy());
    expect(lastSend(phone)).toMatchObject({ text: "For May", files: [{ name: "b.pdf" }] });
  });

  test("over 20 MB or past ten is left out, and says so", async () => {
    const phone = await chat();
    pickFiles([pdf("huge.zip", 20 * 1024 * 1024 + 1)]);
    expect(await screen.findByText("“huge.zip” is over 20 MB.")).toBeTruthy();
    expect(screen.queryByRole("list", { name: "To send" })).toBeNull();

    pickFiles(Array.from({ length: 11 }, (_, n) => pdf(`f${n}.pdf`, 10)));
    expect(await screen.findByText("Up to 10 files in one message.")).toBeTruthy();
    const chosen = await screen.findByRole("list", { name: "To send" });
    expect(within(chosen).getAllByRole("listitem")).toHaveLength(10);
    expect(
      (screen.getByRole("button", { name: "Send a file" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    expect(phone.uploaded).toHaveLength(0);
  });

  test("that the server refuses marks the message, and the file comes back to the box", async () => {
    const phone = await chat();
    phone.uploadFails = "queue_full";
    pickFiles([pdf()]);
    await screen.findByText("receipt.pdf");
    fireEvent.click(sendButton());
    expect(
      await screen.findByText(
        "Not sent. Too many files wait for the computer. Try again once it takes them.",
      ),
    ).toBeTruthy();
    expect(phone.asked.some((ask) => ask.t === "send")).toBe(false);
    const chosen = await screen.findByRole("list", { name: "To send" });
    expect(within(chosen).getByText("receipt.pdf")).toBeTruthy();
  });

  test("a photo this page cannot read goes as it is", async () => {
    const phone = await chat();
    const shot = new File([new Uint8Array(300)], "IMG_1.HEIC", { type: "image/heic" });
    fireEvent.change(screen.getByTestId("photo-picker"), { target: { files: [shot] } });
    await screen.findByText("IMG_1.HEIC");
    fireEvent.click(sendButton());
    await waitFor(() => expect(lastSend(phone)).toBeTruthy());
    expect(lastSend(phone)).toMatchObject({
      files: [{ name: "IMG_1.HEIC", mediaType: "image/heic" }],
    });
  });
});
