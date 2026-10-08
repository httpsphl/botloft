import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeLock, FakePhone } from "./fake";
import { LockScreen } from "./LockScreen";
import { PhoneApp } from "./PhoneApp";
import type { Session } from "./store";
import type { Unlock } from "./vault";

afterEach(cleanup);

const session = { token: "t" } as unknown as Session;

function lockScreen(results: Unlock[]) {
  const unlock = vi.fn(async (_pin: string) => results.shift() as Unlock);
  const forget = vi.fn(async () => {});
  const onOpen = vi.fn();
  const onGone = vi.fn();
  render(<LockScreen unlock={unlock} forget={forget} onOpen={onOpen} onGone={onGone} />);
  return { unlock, forget, onOpen, onGone };
}

const typePin = (value: string) =>
  fireEvent.change(screen.getByLabelText("PIN"), { target: { value } });
const unlockButton = () => screen.getByRole("button", { name: "Unlock" }) as HTMLButtonElement;

describe("the locked page", () => {
  test("opens with the right PIN and takes only numbers", async () => {
    const { unlock, onOpen } = lockScreen([{ ok: session }]);
    expect(unlockButton().disabled).toBe(true);
    typePin("48a2 91x3");
    expect((screen.getByLabelText("PIN") as HTMLInputElement).value).toBe("482913");
    fireEvent.click(unlockButton());
    await waitFor(() => expect(onOpen).toHaveBeenCalledWith(session));
    expect(unlock).toHaveBeenCalledWith("482913");
  });

  test("says how many tries are left, and makes the owner wait after a few wrong ones", async () => {
    lockScreen([
      { wrong: true, left: 9, wait: 0 },
      { wrong: true, left: 1, wait: 30_000 },
    ]);
    typePin("111111");
    fireEvent.click(unlockButton());
    expect(await screen.findByText("Wrong PIN. 9 tries left.")).toBeTruthy();
    expect((screen.getByLabelText("PIN") as HTMLInputElement).value).toBe("");
    typePin("222222");
    fireEvent.click(unlockButton());
    expect(await screen.findByText("Wait 30 s before trying again.")).toBeTruthy();
    // While it waits, the button stays off even with a PIN typed.
    typePin("333333");
    expect(unlockButton().disabled).toBe(true);
  });

  test("after too many wrong ones the phone is gone, and says so", async () => {
    const { onGone } = lockScreen([{ wiped: true }]);
    typePin("111111");
    fireEvent.click(unlockButton());
    expect(await screen.findByText(/Too many wrong PINs/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "OK" }));
    expect(onGone).toHaveBeenCalled();
  });

  test("forgetting the PIN disconnects, after asking", async () => {
    const { forget, onGone } = lockScreen([]);
    fireEvent.click(screen.getByRole("button", { name: "I forgot the PIN" }));
    const dialog = await screen.findByRole("dialog", { name: "Forgot the PIN?" });
    expect(dialog.textContent).toContain("Nothing is lost");
    fireEvent.click(within(dialog).getByRole("button", { name: "Disconnect this phone" }));
    await waitFor(() => expect(forget).toHaveBeenCalled());
    await waitFor(() => expect(onGone).toHaveBeenCalled());
  });
});

describe("the PIN on this phone's page", () => {
  const openSettings = () => fireEvent.click(screen.getByRole("button", { name: "This phone" }));
  const field = (label: string) => screen.getByLabelText(label) as HTMLInputElement;

  test("says why it is worth having and turns on with the same PIN twice", async () => {
    const lock = new FakeLock();
    render(<PhoneApp api={new FakePhone()} lock={lock} />);
    openSettings();
    expect(
      await screen.findByText(
        /whoever picks up this phone unlocked cannot read your conversations/,
      ),
    ).toBeTruthy();
    expect(screen.getByText("The PIN is off.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Turn on the PIN" }));

    // Too short, then different, and only then saved.
    fireEvent.change(field("New PIN (6 to 10 numbers)"), { target: { value: "12345" } });
    fireEvent.change(field("Repeat the PIN"), { target: { value: "12345" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Use 6 to 10 numbers.")).toBeTruthy();
    fireEvent.change(field("New PIN (6 to 10 numbers)"), { target: { value: "482913" } });
    fireEvent.change(field("Repeat the PIN"), { target: { value: "482914" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("The two PINs are different.")).toBeTruthy();
    expect(lock.calls).toEqual([]);
    fireEvent.change(field("Repeat the PIN"), { target: { value: "482913" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText(/The PIN is on/)).toBeTruthy();
    expect(lock.pin).toBe("482913");
  });

  test("turning it off asks for the PIN and refuses a wrong one", async () => {
    const lock = new FakeLock();
    lock.state_ = "on";
    lock.pin = "482913";
    render(<PhoneApp api={new FakePhone()} lock={lock} />);
    openSettings();
    fireEvent.click(await screen.findByRole("button", { name: "Turn off the PIN" }));
    const turnOff = () =>
      fireEvent.click(
        screen.getAllByRole("button", { name: "Turn off the PIN" }).at(-1) as HTMLElement,
      );
    fireEvent.change(field("Current PIN"), { target: { value: "111111" } });
    turnOff();
    expect(await screen.findByText("Wrong PIN.")).toBeTruthy();
    expect(lock.state_).toBe("on");
    fireEvent.change(field("Current PIN"), { target: { value: "482913" } });
    turnOff();
    expect(await screen.findByText("The PIN is off.")).toBeTruthy();
    expect(lock.state_).toBe("off");
  });

  test("a session from before the PIN says to connect again, and offers no button", async () => {
    const lock = new FakeLock();
    lock.state_ = "old";
    render(<PhoneApp api={new FakePhone()} lock={lock} />);
    openSettings();
    expect(await screen.findByText(/connect this phone again/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Turn on the PIN" })).toBeNull();
  });
});

describe("the notice on the inbox", () => {
  const nudge = "Protect this phone with a PIN?";

  test("gives the reason, and does not come back after not now", async () => {
    const lock = new FakeLock();
    const first = render(<PhoneApp api={new FakePhone()} lock={lock} />);
    expect(await screen.findByText(nudge)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Not now" }));
    await waitFor(() => expect(screen.queryByText(nudge)).toBeNull());
    expect(lock.dismissed).toBe(true);
    first.unmount();

    // Opened again, it stays quiet.
    render(<PhoneApp api={new FakePhone()} lock={lock} />);
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(screen.queryByText(nudge)).toBeNull();
  });

  test("yes takes the owner to the PIN", async () => {
    render(<PhoneApp api={new FakePhone()} lock={new FakeLock()} />);
    fireEvent.click(await screen.findByRole("button", { name: "Protect with a PIN" }));
    expect(await screen.findByText("The PIN is off.")).toBeTruthy();
  });

  test("is not there when the PIN is on, or when the page has no lock to offer", async () => {
    const on = new FakeLock();
    on.state_ = "on";
    render(<PhoneApp api={new FakePhone()} lock={on} />);
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(screen.queryByText(nudge)).toBeNull();
    cleanup();
    render(<PhoneApp api={new FakePhone()} />);
    expect(screen.queryByText(nudge)).toBeNull();
  });
});
