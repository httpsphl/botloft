import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost, FakeUpdate } from "../../lib/fakeHost";

afterEach(cleanup);

function renderApp(host: FakeHost) {
  const fake = new FakeBotloft();
  fake.addCrew("Ops");
  render(<App host={host} connect={() => fake as Client} />);
}

describe("app updates", () => {
  test("nothing shows while this is the latest version", async () => {
    const host = new FakeHost();
    renderApp(host);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    expect(host.updateChecks).toBe(1);
    expect(screen.queryByRole("button", { name: "Update available" })).toBeNull();
  });

  test("a new version installs in one click, in plain words", async () => {
    const host = new FakeHost();
    const update = new FakeUpdate("0.2.0");
    update.notes = "Bots start faster.";
    host.update = update;
    renderApp(host);
    fireEvent.click(await screen.findByRole("button", { name: "Update available" }));
    const dialog = screen.getByRole("dialog", { name: "Update Botloft" });
    expect(dialog.textContent).toContain("Botloft 0.2.0 is ready");
    expect(dialog.textContent).toContain("Bots start faster.");
    fireEvent.click(within(dialog).getByRole("button", { name: "Update now" }));
    expect(await within(dialog).findByText("Downloading… 50%")).toBeDefined();
    expect(update.installs).toBe(1);
    expect(
      (within(dialog).getByRole("button", { name: "Later" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  test("a failed update says so and can be tried again", async () => {
    const host = new FakeHost();
    const update = new FakeUpdate("0.2.0");
    update.failure = new Error("signature verification failed");
    host.update = update;
    renderApp(host);
    fireEvent.click(await screen.findByRole("button", { name: "Update available" }));
    const dialog = screen.getByRole("dialog", { name: "Update Botloft" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Update now" }));
    expect(await within(dialog).findByText("The update didn't install")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Try again" }));
    expect(update.installs).toBe(2);
  });

  test("an unreachable release feed stays quiet", async () => {
    const host = new FakeHost();
    host.update = new Error("offline");
    renderApp(host);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    expect(screen.queryByRole("button", { name: "Update available" })).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
