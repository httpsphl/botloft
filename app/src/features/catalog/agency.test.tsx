import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";

afterEach(() => {
  cleanup();
  setLocaleChoice("system");
});

/** A crew "Site" with only its chief, open on its page. */
async function openCrewWithChief() {
  const fake = new FakeBotloft();
  await fake.call("crews.create", {
    name: "Site",
    lead: { name: "Chief", role: "Leads the crew", instructions: "Build the site" },
  });
  renderApp(fake);
  await crewOpened("Site");
  return fake;
}

const addCalls = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "catalog.add").map((call) => call.params);

describe("the Agent agency", () => {
  test("the invitation shows a few agents, and a button for all of them", async () => {
    await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agent agency" });
    expect(await within(invite).findAllByRole("listitem")).toHaveLength(8);
    expect(within(invite).queryByRole("searchbox")).toBeNull();

    fireEvent.click(within(invite).getByRole("button", { name: "See all agents" }));
    const dialog = await screen.findByRole("dialog", { name: "Agent agency" });
    expect(await within(dialog).findAllByRole("listitem")).toHaveLength(129);
  });

  test("a crew with only its chief invites the owner to pick agents, and a click adds one", async () => {
    const fake = await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agent agency" });
    expect(within(invite).getByText("Who do you want on your crew?")).toBeDefined();

    fireEvent.click(within(invite).getByRole("button", { name: "Add to crew: Designer" }));
    expect(await within(invite).findByText("Designer joined the crew")).toBeDefined();
    expect(addCalls(fake)).toEqual([
      {
        crewId: expect.any(String),
        templateId: "designer",
        name: "Designer",
        role: "Designs screens and pages in HTML that you see live and can ask to change",
      },
    ]);
    expect(within(sidebar()).getByRole("button", { name: /^Designer/ })).toBeDefined();
  });

  test("the invitation stays while the page is open, and is not there for a crew that has a team", async () => {
    const fake = await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agent agency" });
    fireEvent.click(within(invite).getByRole("button", { name: "Add to crew: Writer" }));
    await within(invite).findByText("Writer joined the crew");
    fireEvent.click(within(invite).getByRole("button", { name: "Add to crew: Designer" }));
    await within(invite).findByText("Designer joined the crew");

    // Opening the crew again, with a team, shows no invitation.
    const other = await fake.call("crews.create", { name: "Blog" });
    for (const name of ["Ana", "Bia"]) {
      await fake.call("bots.create", {
        crewId: other.id,
        name,
        role: "",
        instructions: "",
      });
    }
    fireEvent.click(within(sidebar()).getAllByRole("button", { name: /^Blog/ })[0] as HTMLElement);
    await crewOpened("Blog");
    await act(async () => {});
    expect(screen.queryByRole("region", { name: "Agent agency" })).toBeNull();
  });

  test("the crew's button opens the whole agency, with filters and search", async () => {
    await openCrewWithChief();
    fireEvent.click(screen.getByRole("button", { name: "Agent agency" }));
    const dialog = await screen.findByRole("dialog", { name: "Agent agency" });
    expect(await within(dialog).findAllByRole("listitem")).toHaveLength(129);

    fireEvent.click(within(dialog).getByRole("button", { name: "Marketing & sales" }));
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(24);
    fireEvent.click(within(dialog).getByRole("button", { name: "Product & management" }));
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(14);

    fireEvent.click(within(dialog).getByRole("button", { name: "Research" }));
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(4);
    expect(
      within(dialog).getByRole("button", { name: "Research" }).getAttribute("aria-pressed"),
    ).toBe("true");

    fireEvent.click(within(dialog).getByRole("button", { name: "All" }));
    fireEvent.change(within(dialog).getByRole("searchbox", { name: "Search agents" }), {
      target: { value: "spreadsheets" },
    });
    const found = within(dialog).getAllByRole("listitem");
    expect(found).toHaveLength(1);
    expect(within(found[0] as HTMLElement).getByText("Data Analyst")).toBeDefined();

    fireEvent.change(within(dialog).getByRole("searchbox", { name: "Search agents" }), {
      target: { value: "zzz" },
    });
    expect(within(dialog).getByText("No agent matches that.")).toBeDefined();
  }, 20_000);

  test("Learn more shows what the agent does and what it is told, and adds it from there", async () => {
    const fake = await openCrewWithChief();
    fireEvent.click(screen.getByRole("button", { name: "Agent agency" }));
    const dialog = await screen.findByRole("dialog", { name: "Agent agency" });
    fireEvent.click(
      await within(dialog).findByRole("button", { name: "Learn more: Code Reviewer" }),
    );

    const detail = await within(dialog).findByRole("region", { name: "Code Reviewer" });
    expect(within(detail).getByText("When to call it")).toBeDefined();
    expect(within(detail).getByText("Works well with")).toBeDefined();
    const technical = await within(detail).findByText("What the agent is told when it starts");
    expect(technical.closest("details")?.textContent).toContain("### What you do");
    expect(technical.closest("details")?.textContent).toContain("high");

    fireEvent.click(within(detail).getByRole("button", { name: "Add to crew" }));
    expect(await within(detail).findByText("Code Reviewer joined the crew")).toBeDefined();
    expect(addCalls(fake)[0]).toMatchObject({ templateId: "code-reviewer" });

    fireEvent.click(within(dialog).getByRole("button", { name: "Back to all agents" }));
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(129);
  });

  test("the same role twice gets another name", async () => {
    const fake = await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agent agency" });
    const add = within(invite).getByRole("button", { name: "Add to crew: Designer" });
    fireEvent.click(add);
    await within(invite).findByText("Designer joined the crew");
    fireEvent.click(add);
    await within(invite).findByText("Designer 2 joined the crew");
    expect(addCalls(fake).map((params) => (params as { name: string }).name)).toEqual([
      "Designer",
      "Designer 2",
    ]);
  });

  test("Customize opens the new agent", async () => {
    await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agent agency" });
    fireEvent.click(within(invite).getByRole("button", { name: "Add to crew: Designer" }));
    await within(invite).findByText("Designer joined the crew");
    fireEvent.click(within(invite).getByRole("button", { name: "Customize" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Designer" })).toBeDefined();
  });

  test("the owner reads it in their language", async () => {
    setLocaleChoice("pt-BR");
    const fake = await openCrewWithChief();
    const invite = await screen.findByRole("region", { name: "Agência de agentes" });
    expect(within(invite).getByText("Quem você quer na sua equipe?")).toBeDefined();
    fireEvent.click(within(invite).getByRole("button", { name: "Adicionar na equipe: Redator" }));
    expect(await within(invite).findByText("Redator entrou na equipe")).toBeDefined();
    expect(addCalls(fake)[0]).toMatchObject({ templateId: "writer", name: "Redator" });
  });
});
