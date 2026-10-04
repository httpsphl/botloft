import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

const PROJECT = "D:\\Projects\\Bakery site";

describe("work folders", () => {
  test("a new crew can work in a folder the owner picks", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    const host = new FakeHost();
    host.nextFolder = PROJECT;
    renderApp(fake, host);
    await crewOpened("Ops");

    fireEvent.click(screen.getByRole("button", { name: "New crew" }));
    const dialog = screen.getByRole("dialog", { name: "New crew" });
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Bakery" } });
    expect(within(dialog).getByText("A new folder inside Botloft")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose folder…" }));
    expect(await within(dialog).findByText(PROJECT)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    // The new crew opens on its chief's chat.
    await screen.findByRole("heading", { level: 1, name: "Chief" });
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).toMatchObject({ name: "Bakery", workFolder: PROJECT });
    const bakery = [...fake.crews.values()].find((crew) => crew.name === "Bakery");
    expect(bakery?.workFolder).toBe(PROJECT);
  });

  test("without a choice the crew gets a new folder", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    const host = new FakeHost();
    host.nextFolder = PROJECT;
    renderApp(fake, host);
    await crewOpened("Ops");

    fireEvent.click(screen.getByRole("button", { name: "New crew" }));
    const dialog = screen.getByRole("dialog", { name: "New crew" });
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Bakery" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose folder…" }));
    await within(dialog).findByText(PROJECT);
    fireEvent.click(within(dialog).getByRole("button", { name: "Use a new folder" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    await screen.findByRole("heading", { level: 1, name: "Chief" });
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).not.toHaveProperty("workFolder");
    const bakery = [...fake.crews.values()].find((crew) => crew.name === "Bakery");
    expect(bakery?.workFolderChosen).toBe(false);
  });

  test("the owner opens the folder and moves the crew to another one", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const host = new FakeHost();
    renderApp(fake, host);
    await crewOpened("Ops");

    // Only the icon: the path is in the tooltip.
    const folder = screen.getByRole("button", { name: `Open work folder: ${ops.workFolder}` });
    expect(folder.textContent).toBe("");
    expect(folder.title).toBe(`Works in ${ops.workFolder}
Click to open it`);
    fireEvent.click(folder);
    expect(host.opened).toEqual([ops.workFolder]);

    host.nextFolder = PROJECT;
    fireEvent.click(screen.getByRole("button", { name: "More crew actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Change work folder…" }));
    const confirm = await screen.findByRole("dialog", { name: "Move Ops to another folder?" });
    expect(host.pickerStarts).toEqual([ops.workFolder]);
    expect(within(confirm).getByText(/restarts when it finishes/)).toBeDefined();
    fireEvent.click(within(confirm).getByRole("button", { name: "Move" }));

    expect(
      await screen.findByRole("button", { name: `Open work folder: ${PROJECT}` }),
    ).toBeDefined();
    const move = fake.calls.find((call) => call.method === "crews.setWorkFolder");
    expect(move?.params).toEqual({ crewId: ops.id, workFolder: PROJECT });
  });

  test("cancelling the picker changes nothing", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    const host = new FakeHost();
    host.nextFolder = null;
    renderApp(fake, host);
    await crewOpened("Ops");
    fireEvent.click(screen.getByRole("button", { name: "More crew actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Change work folder…" }));
    await Promise.resolve();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(fake.calls.some((call) => call.method === "crews.setWorkFolder")).toBe(false);
  });
});
