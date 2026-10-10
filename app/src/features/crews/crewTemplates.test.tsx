import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FAKE_CATALOG } from "../../lib/fakeCatalog";
import { RpcError } from "../../lib/rpc";
import { crewOpened, renderApp, sidebar } from "../../test/app";
import { CREW_TEMPLATES } from "./crewTemplates";

afterEach(cleanup);

/** The app with one crew open, and the template step of "New crew" showing. */
async function openTemplates() {
  const fake = new FakeBotloft();
  fake.addCrew("Ops");
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(screen.getByRole("button", { name: "New crew" }));
  return { fake, chooser: await screen.findByRole("dialog", { name: "Start from a template" }) };
}

describe("the crew templates", () => {
  test("every role of every template is a role of the catalog, and a crew has room for it", () => {
    const known = new Set(FAKE_CATALOG.map((role) => role.id));
    for (const template of CREW_TEMPLATES) {
      for (const id of template.roles) {
        expect(known.has(id), `${template.id}: ${id}`).toBe(true);
      }
      expect(new Set(template.roles).size).toBe(template.roles.length);
      // The chief takes one of the 12 places of a crew.
      expect(template.roles.length).toBeLessThanOrEqual(11);
    }
    expect(new Set(CREW_TEMPLATES.map((template) => template.id)).size).toBe(CREW_TEMPLATES.length);
  });

  test("the first step lists the templates and an empty crew", async () => {
    const { chooser } = await openTemplates();
    expect(within(chooser).getAllByRole("listitem")).toHaveLength(CREW_TEMPLATES.length);
    expect(within(chooser).getByRole("button", { name: /Software team/ })).toBeDefined();
    expect(within(chooser).getByRole("button", { name: /Start with an empty crew/ })).toBeDefined();
  });

  test("picking a template fills the form and says which agents it adds", async () => {
    const { chooser } = await openTemplates();
    fireEvent.click(within(chooser).getByRole("button", { name: /Game studio/ }));

    const dialog = screen.getByRole("dialog", { name: "New crew" });
    expect((within(dialog).getByLabelText("Name") as HTMLInputElement).value).toBe("Game studio");
    expect(
      (within(dialog).getByLabelText("What is this crew for?") as HTMLTextAreaElement).value,
    ).toContain("Design my game");
    expect(
      within(dialog).getByText(/Game Designer, Narrative Designer, Level Designer/),
    ).toBeDefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "Pick another template" }));
    expect(screen.getByRole("dialog", { name: "Start from a template" })).toBeDefined();
  });

  test("creating from a template makes the crew with its chief and then each agent", async () => {
    const { fake, chooser } = await openTemplates();
    fireEvent.click(within(chooser).getByRole("button", { name: /Online store/ }));
    const dialog = screen.getByRole("dialog", { name: "New crew" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    await crewOpened("Online store");
    const template = CREW_TEMPLATES.find((item) => item.id === "online-store");
    await waitFor(() =>
      expect(fake.calls.filter((call) => call.method === "catalog.add")).toHaveLength(
        template?.roles.length ?? -1,
      ),
    );
    expect(
      fake.calls.filter((call) => call.method === "catalog.add").map((call) => call.params),
    ).toEqual((template?.roles ?? []).map((id) => expect.objectContaining({ templateId: id })));
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).toMatchObject({ name: "Online store", lead: { name: "Chief" } });
    expect(await within(sidebar()).findByRole("button", { name: /^Bookkeeper/ })).toBeDefined();
  });

  test("an agent that cannot be added is told, and the crew stays", async () => {
    const { fake, chooser } = await openTemplates();
    fireEvent.click(within(chooser).getByRole("button", { name: /Research desk/ }));
    fake.failNext("catalog.add", new RpcError(-32603, "no room"));
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "New crew" })).getByRole("button", {
        name: "Create crew",
      }),
    );

    expect(await screen.findByText(/1 agent could not be added/)).toBeDefined();
    expect(fake.calls.filter((call) => call.method === "catalog.add")).toHaveLength(6);
    expect(await screen.findByRole("heading", { level: 1, name: "Chief" })).toBeDefined();
  });
});
