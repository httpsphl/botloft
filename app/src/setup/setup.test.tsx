import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeSetupHost, type FakeSetupOptions } from "./fakeSetupHost";
import { asFailure } from "./host";
import { Setup } from "./Setup";

afterEach(cleanup);

async function open(options: FakeSetupOptions = {}) {
  const host = new FakeSetupHost(options);
  render(<Setup host={host} openAfter={0} />);
  await screen.findByRole("heading");
  return host;
}

const button = (name: string) => screen.queryByRole("button", { name });

describe("setup window", () => {
  test("installs, then opens the app", async () => {
    const host = await open();
    expect(screen.getByRole("heading", { name: "Botloft" })).toBeDefined();
    expect(screen.getByText(/without asking for an administrator/)).toBeDefined();
    expect(host.calls).toContain("show");

    fireEvent.click(screen.getByRole("button", { name: "Install" }));
    expect(screen.getByRole("progressbar")).toBeDefined();
    // Nothing closes the window halfway through.
    expect(button("Close")).toBeNull();

    await screen.findByText("All set.");
    await waitFor(() => expect(host.calls).toContain("openApp"));
  });

  test("updates an older version", async () => {
    await open({ relation: "older", installed: "0.5.0" });
    expect(screen.getByText("You have 0.5.0. This is 0.6.0.")).toBeDefined();
    expect(button("Update")).not.toBeNull();
    expect(button("Install")).toBeNull();
  });

  test("the same version opens it, or installs again", async () => {
    const host = await open({ relation: "same", installed: "0.6.0" });
    expect(screen.getByText("Botloft is already installed.")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Install again" }));
    await screen.findByText("All set.");
    expect(host.calls).toContain("install");
  });

  test("a newer version is only opened", async () => {
    const host = await open({ relation: "newer", installed: "0.9.0" });
    expect(screen.getByText("You already have a newer version (0.9.0).")).toBeDefined();
    expect(button("Install")).toBeNull();
    expect(button("Install again")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Open Botloft" }));
    expect(host.calls).toContain("openApp");
    expect(host.calls).not.toContain("install");
  });

  test("says the open app closes and the bots keep working", async () => {
    await open({ relation: "older", appRunning: true });
    expect(screen.getByText(/closes to install\. Your bots keep working/)).toBeDefined();
  });

  test("a failed install offers to retry or the classic installer", async () => {
    const host = await open({ failure: { code: 2, detail: "installer exited with 2" } });
    fireEvent.click(screen.getByRole("button", { name: "Install" }));
    await screen.findByText("Botloft didn't install.");
    expect(screen.getByText(/Exit code: 2/)).toBeDefined();
    expect(button("Close")).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Use the classic installer" }));
    expect(host.calls).toContain("classic");

    host.succeed();
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    await screen.findByText("All set.");
  });

  test("tells where to open the app when opening it fails", async () => {
    const host = new FakeSetupHost();
    host.openApp = async () => {
      throw new Error("cannot open Botloft.exe");
    };
    render(<Setup host={host} openAfter={0} />);
    fireEvent.click(await screen.findByRole("button", { name: "Install" }));
    await screen.findByText("Botloft is installed. Open it from the Start menu.");
  });

  test("closes from its own title strip", async () => {
    const host = await open();
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Close" })));
    expect(host.calls).toContain("close");
  });
});

describe("install failures", () => {
  test("keeps the exit code and detail, or makes do with any rejection", () => {
    expect(asFailure({ code: 3, detail: "boom" })).toEqual({ code: 3, detail: "boom" });
    expect(asFailure({ code: null, detail: "no code" })).toEqual({ code: null, detail: "no code" });
    expect(asFailure("plain text")).toEqual({ code: null, detail: "plain text" });
  });
});
