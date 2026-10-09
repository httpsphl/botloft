import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { renderApp } from "../../test/app";

afterEach(cleanup);

describe("ideas on the welcome screen", () => {
  test("tapping an idea opens a new crew already set up from that template", async () => {
    renderApp(new FakeBotloft());
    const ideas = await screen.findByRole("region", { name: "Or tap an idea" });
    expect(within(ideas).getAllByRole("listitem")).toHaveLength(4);

    fireEvent.click(within(ideas).getByRole("button", { name: /Software team/ }));
    const dialog = await screen.findByRole("dialog", { name: "New crew" });
    expect((within(dialog).getByLabelText("Name") as HTMLInputElement).value).toBe("Software team");
    expect(within(dialog).getByText(/This team adds/)).toBeDefined();
  });

  test("the plain button still starts at the list of templates", async () => {
    renderApp(new FakeBotloft());
    fireEvent.click(await screen.findByRole("button", { name: "Create your first crew" }));
    expect(await screen.findByRole("dialog", { name: "Start from a template" })).toBeDefined();
  });
});
