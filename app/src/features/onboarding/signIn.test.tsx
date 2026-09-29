import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";

afterEach(cleanup);

function renderApp(fake: FakeBotloft, host = new FakeHost()) {
  render(<App host={host} connect={() => fake as Client} />);
  return host;
}

function signedOut(): FakeBotloft {
  const fake = new FakeBotloft();
  fake.system = { ...fake.system, claudeSignedIn: false };
  return fake;
}

describe("signing in to Claude", () => {
  test("the welcome screen says the account is ready", async () => {
    renderApp(new FakeBotloft());
    expect(await screen.findByText("Claude account")).toBeDefined();
    expect(screen.getByText("Signed in.")).toBeDefined();
  });

  test("one click opens Claude's sign-in and the app sees it finish", async () => {
    const fake = signedOut();
    const host = new FakeHost();
    host.onSignIn = () => {
      fake.system = { ...fake.system, claudeSignedIn: true };
    };
    renderApp(fake, host);
    fireEvent.click(await screen.findByRole("button", { name: "Sign in to Claude" }));
    expect(await screen.findByText("Signed in.")).toBeDefined();
    expect(host.signIns).toEqual([fake.system.claudePath]);
    expect(fake.refreshes).toBe(1);
  });

  test("a sign-in window closed early says so and can be tried again", async () => {
    const host = new FakeHost();
    host.signInResult = false;
    renderApp(signedOut(), host);
    fireEvent.click(await screen.findByRole("button", { name: "Sign in to Claude" }));
    expect(await screen.findByText("The sign-in didn't finish")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Sign in to Claude" }));
    expect(host.signIns).toHaveLength(2);
  });

  test("with crews, a banner asks to sign in", async () => {
    const fake = signedOut();
    fake.addCrew("Ops");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    const banner = screen.getByText("Sign in to Claude", { selector: "p" }).closest("[role=alert]");
    expect(
      within(banner as HTMLElement).getByRole("button", { name: "Sign in to Claude" }),
    ).toBeDefined();
  });

  test("a bot stopped by a sign-in error offers the sign-in", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await screen.findByRole("heading", { level: 1, name: "Ops" });
    const sidebar = screen.getByRole("navigation", { name: "Crews" });
    fireEvent.click(within(sidebar).getByRole("button", { name: /Scout/ }));
    act(() => fake.setBotState(bot.id, "auth_error"));
    const view = screen.getByRole("region", { name: "Scout" });
    expect(within(view).getByRole("button", { name: "Sign in to Claude" })).toBeDefined();
  });
});
