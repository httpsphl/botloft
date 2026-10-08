import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { ApprovalCard, QuestionCard } from "../lib/protocol.gen";
import { FakePhone } from "./fake";
import { guessName, PairScreen } from "./PairScreen";
import { PhoneApp } from "./PhoneApp";
import { type Pair, PairError } from "./pair";
import type { Session } from "./store";

afterEach(cleanup);

const approval = (change: Partial<ApprovalCard> = {}): ApprovalCard => ({
  approvalId: "apr_1",
  bot: { name: "Scout", color: "#aabbcc" },
  crew: "Ops",
  createdAt: Date.now(),
  toolName: "Bash",
  summary: "git status",
  explanation: "Shows what changed",
  text: "git status --short",
  cut: false,
  atComputer: false,
  ...change,
});

const question = (change: Partial<QuestionCard> = {}): QuestionCard => ({
  questionId: "qst_1",
  bot: { name: "Writer", color: "#ccbbaa" },
  crew: "Ops",
  createdAt: Date.now(),
  text: "Which **client** first?",
  options: ["Acme", "Globex"],
  ...change,
});

describe("what waits for the owner", () => {
  test("says when it is looking, and when nothing waits", () => {
    const phone = new FakePhone();
    phone.set({ loaded: false });
    render(<PhoneApp api={phone} />);
    expect(screen.getByText("Checking what is waiting…")).toBeTruthy();
    act(() => phone.set({ loaded: true }));
    expect(screen.getByText("Nothing is waiting for you")).toBeTruthy();
  });

  test("shows a request with what the bot says it is for, and the whole of it a tap away", async () => {
    const phone = new FakePhone();
    phone.add(approval());
    render(<PhoneApp api={phone} />);
    expect(screen.getByText("Scout asks to")).toBeTruthy();
    expect(screen.getByText("git status")).toBeTruthy();
    expect(screen.getByText("Shows what changed")).toBeTruthy();
    expect(screen.getByText("Scout wrote this")).toBeTruthy();
    expect(screen.queryByText("git status --short")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Show the whole request" }));
    expect(screen.getByText("git status --short")).toBeTruthy();
  });

  test("warns when the bot did not say what a command is for", () => {
    const phone = new FakePhone();
    const { explanation: _unsaid, ...unexplained } = approval();
    phone.add(unexplained);
    render(<PhoneApp api={phone} />);
    expect(screen.getByText(/Scout did not say what this is for/)).toBeTruthy();
  });

  test("allows with a note, denies, and says when it could not send", async () => {
    const phone = new FakePhone();
    phone.add(approval());
    render(<PhoneApp api={phone} />);
    fireEvent.change(screen.getByLabelText("Note for the bot (optional)"), {
      target: { value: "go on" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Allow" }));
    await waitFor(() =>
      expect(phone.calls).toEqual([{ method: "answerApproval", args: ["apr_1", true, "go on"] }]),
    );

    phone.sendable = false;
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));
    expect(await screen.findByText("Not sent. Check the connection and try again.")).toBeTruthy();
    expect(phone.calls[1]?.args).toEqual(["apr_1", false, "go on"]);
  });

  test("a request that is cut, or belongs to the computer, can only be denied", () => {
    const phone = new FakePhone();
    phone.add(approval({ approvalId: "apr_cut", cut: true }));
    phone.add(approval({ approvalId: "apr_pc", atComputer: true }));
    render(<PhoneApp api={phone} />);
    for (const allow of screen.getAllByRole("button", { name: "Allow" })) {
      expect((allow as HTMLButtonElement).disabled).toBe(true);
    }
    for (const deny of screen.getAllByRole("button", { name: "Deny" })) {
      expect((deny as HTMLButtonElement).disabled).toBe(false);
    }
    expect(screen.getByText(/Too long to read on a phone/)).toBeTruthy();
    expect(screen.getByText(/needs you at the computer/)).toBeTruthy();
  });

  test("a request on its way cannot be pressed twice", () => {
    const phone = new FakePhone();
    phone.add(approval());
    phone.set({ sending: ["apr_1"] });
    render(<PhoneApp api={phone} />);
    for (const button of [
      screen.getByRole("button", { name: "Allow" }),
      screen.getByRole("button", { name: "Deny" }),
    ]) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
    }
  });

  test("answers a question with a ready answer or with words, or dismisses it", async () => {
    const phone = new FakePhone();
    phone.add(undefined, question());
    render(<PhoneApp api={phone} />);
    // The text is markdown: the emphasis is not shown as asterisks.
    expect(screen.getByText("client").tagName).toBe("STRONG");
    fireEvent.click(screen.getByRole("button", { name: "Globex" }));
    fireEvent.change(screen.getByLabelText("Your answer"), { target: { value: "  Initech " } });
    fireEvent.click(screen.getByRole("button", { name: "Answer" }));
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    await waitFor(() => expect(phone.calls.length).toBe(3));
    expect(phone.calls).toEqual([
      { method: "answerQuestion", args: ["qst_1", "Globex"] },
      { method: "answerQuestion", args: ["qst_1", "Initech"] },
      { method: "dismissQuestion", args: ["qst_1"] },
    ]);
  });

  test("lists the oldest first and says when the computer is off or the connection is lost", () => {
    const phone = new FakePhone();
    phone.add(approval());
    phone.set({ computer: "offline", link: "offline" });
    render(<PhoneApp api={phone} />);
    expect(screen.getByText(/Your computer is off or has no internet/)).toBeTruthy();
    expect(screen.getByText("No connection. Trying again…")).toBeTruthy();
  });
});

describe("this phone", () => {
  test("disconnecting asks first, then leaves", async () => {
    const phone = new FakePhone();
    render(<PhoneApp api={phone} />);
    fireEvent.click(screen.getByRole("button", { name: "This phone" }));
    expect(screen.getByText("Celular da Ana", { exact: false })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Disconnect this phone" }));
    const dialog = await screen.findByRole("dialog", { name: "Disconnect this phone?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Disconnect this phone" }));
    await waitFor(() => expect(phone.calls.map((call) => call.method)).toEqual(["disconnect"]));
    expect(await screen.findByText("Phone disconnected")).toBeTruthy();
  });

  test("a phone the computer cut off says so and forgets itself on OK", async () => {
    const phone = new FakePhone();
    phone.set({ session: "revoked" });
    render(<PhoneApp api={phone} />);
    expect(screen.getByText("This phone was disconnected")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "OK" }));
    expect(phone.calls.map((call) => call.method)).toEqual(["disconnect"]);
  });
});

describe("connecting this phone", () => {
  const session = { token: "t", device: "d", peer: "p", name: "Phone" } as unknown as Session;
  const pairOf = (join: Pair["join"]): Pair => ({ join });

  test("explains how to get here when the page was not opened from a code", () => {
    render(<PairScreen pair={null} onSession={() => {}} />);
    expect(screen.getByText(/point its camera at the code on your computer/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Connect" })).toBeNull();
  });

  test("takes a name, shows the six digits to compare and hands over the session", async () => {
    const onSession = vi.fn();
    const join = vi.fn(async (name: string) => ({
      code: "482913",
      collect: async () => ({ ...session, name }),
    }));
    render(<PairScreen pair={pairOf(join)} onSession={onSession} />);
    fireEvent.change(screen.getByLabelText("Name of this phone"), {
      target: { value: "Celular da Ana" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Connect" }));
    expect(await screen.findByText("482 913")).toBeTruthy();
    expect(screen.getByText(/Check that your computer shows 482 913 too/)).toBeTruthy();
    await waitFor(() =>
      expect(onSession).toHaveBeenCalledWith({ ...session, name: "Celular da Ana" }),
    );
    expect(join).toHaveBeenCalledWith("Celular da Ana");
  });

  test("says why it could not connect, and lets the owner try again", async () => {
    const join = vi.fn(async () => {
      throw new PairError("expired");
    });
    render(<PairScreen pair={pairOf(join)} onSession={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: "Connect" }));
    expect(await screen.findByText(/ran out or was already used/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Connect" })).toBeTruthy();
  });

  test("starts from a name that fits the kind of phone", () => {
    const names = { iphone: "iPhone", ipad: "iPad", android: "Android phone", other: "Phone" };
    expect(guessName(names, "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)")).toBe("iPhone");
    expect(guessName(names, "Mozilla/5.0 (iPad; CPU OS 17_0)")).toBe("iPad");
    expect(guessName(names, "Mozilla/5.0 (Linux; Android 14; Pixel 8)")).toBe("Android phone");
    expect(guessName(names, "Mozilla/5.0 (X11; Linux x86_64)")).toBe("Phone");
  });
});
