import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FAKE_PASSPHRASE } from "../../lib/fakeBackup";
import { FakeHost } from "../../lib/fakeHost";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

async function openBackup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  fake.addBot(ops.id, "Writer");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Backup" }));
  return { fake, host, dialog };
}

const type = (field: HTMLElement, value: string) => fireEvent.change(field, { target: { value } });

describe("backup", () => {
  test("a copy is saved where the owner picks, once both passphrases match", async () => {
    const { fake, host, dialog } = await openBackup();
    const save = within(dialog).getByRole("button", { name: "Save a copy…" });
    expect(save).toHaveProperty("disabled", true);
    type(within(dialog).getByLabelText("Passphrase"), "correct horse");
    type(within(dialog).getByLabelText("Passphrase again"), "correct hors");
    expect(within(dialog).getByText("The two passphrases are not the same.")).toBeDefined();
    expect(save).toHaveProperty("disabled", true);
    type(within(dialog).getByLabelText("Passphrase again"), "correct horse");
    host.saveChosen = true;
    fireEvent.click(save);
    expect(await within(dialog).findByText(/^Copy saved \(/)).toBeDefined();
    expect(host.savedFiles[0]).toMatch(/\.botloft$/);
    const call = fake.calls.find((each) => each.method === "backup.export");
    expect(call?.params).toEqual({ passphrase: "correct horse" });
  });

  test("a copy opens with its passphrase, shows what it holds and restores on restart", async () => {
    const { fake, host, dialog } = await openBackup();
    host.nextFile = "D:\\Copies\\botloft-2026-10-05-1430.botloft";
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose a copy…" }));
    const field = await within(dialog).findByLabelText("The copy's passphrase");
    expect(host.fileFilters[0]).toEqual(["botloft"]);

    type(field, "wrong horse");
    fireEvent.click(within(dialog).getByRole("button", { name: "Open" }));
    expect(
      await within(dialog).findByText("The passphrase is wrong, or the file is damaged."),
    ).toBeDefined();

    type(field, FAKE_PASSPHRASE);
    fireEvent.click(within(dialog).getByRole("button", { name: "Open" }));
    expect(await within(dialog).findByText("Ops: 2 agents")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Restore and restart" }));
    await waitFor(() => expect(host.installs).toContain("restart"));
    expect(fake.calls.some((each) => each.method === "backup.confirm")).toBe(true);
  });
});
