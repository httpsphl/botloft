import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FAKE_BACKUP_BYTES, FAKE_PASSPHRASE } from "../../lib/fakeBackup";
import { FakeHost } from "../../lib/fakeHost";
import { fakeUploads } from "../../lib/fakeUploads";
import type { BotFile } from "../../lib/protocol.gen";
import { server } from "../../shell/server";
import { crewOpened, openBot, renderApp } from "../../test/app";

// The app on a server's Botloft (spec 42.7): files come and go in pieces
// through the connection, and this computer's folders stay out.

const vps = { name: "vps-1", port: 45710, tunnel: 45720, token: "device-token" };

afterEach(() => {
  cleanup();
  server.clear();
});

const text = (bytes: number[]) => new TextDecoder().decode(new Uint8Array(bytes));

/** A crew "Ops" with @scout, idle, the app on the server. */
function onServer() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  server.set(vps);
  return { fake, scout };
}

/** Scout shares `files` with `share_file`. */
function share(fake: FakeBotloft, botId: string, files: BotFile[]) {
  const call = fake.chat.tool(botId, "mcp__botloft__share_file", {
    summary: files.map((file) => file.name).join(", "),
    input: JSON.stringify({ files: files.map((file) => file.path) }),
  });
  fake.chat.finish(call, JSON.stringify({ shown: files }), false);
}

describe("files from a server", () => {
  test("a shared file is saved and opened through the connection", async () => {
    const { fake, scout } = onServer();
    const report = fake.files.add(scout.id, "report.pdf", { text: "%PDF-1.7" });
    share(fake, scout.id, [report]);
    const host = new FakeHost();
    renderApp(fake, host);
    await crewOpened("Ops");
    openBot("Scout");
    const list = await screen.findByRole("list", { name: "Files shared by Scout" });
    const card = within(within(list).getAllByRole("listitem")[0] as HTMLElement);

    fireEvent.click(card.getByRole("button", { name: "Save as…" }));
    await waitFor(() => expect(host.transfers.downloads[0]?.kept).toBe(true));
    const saved = host.transfers.downloads[0];
    expect(saved?.name).toBe("report.pdf");
    expect(saved?.temp).toBe(false);
    expect(text(saved?.bytes ?? [])).toBe("%PDF-1.7");

    fireEvent.click(card.getByRole("button", { name: "Open" }));
    await waitFor(() => expect(host.transfers.downloads[1]?.opened).toBe(true));
    expect(host.transfers.downloads[1]?.temp).toBe(true);
    expect(host.savedFiles).toEqual([]);
    expect(host.openedFiles).toEqual([]);
  });

  test("this computer's folders do not open", async () => {
    const { fake } = onServer();
    renderApp(fake, new FakeHost());
    await crewOpened("Ops");
    expect(screen.queryByRole("button", { name: /^Open work folder/ })).toBeNull();
  });

  test("a backup to restore goes to the server first", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    server.set(vps);
    const host = new FakeHost();
    const bytes = new TextEncoder().encode(FAKE_BACKUP_BYTES);
    host.transfers.nextPick = { name: "copy.botloft", bytes };
    renderApp(fake, host);
    await crewOpened("Ops");
    fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
    const dialog = screen.getByRole("region", { name: "Settings" });
    fireEvent.click(within(dialog).getByRole("tab", { name: "Backup" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose a copy…" }));

    const path = "/var/lib/botloft/home/uploads/up1/copy.botloft";
    expect(await within(dialog).findByText(path)).toBeTruthy();
    const sent = fakeUploads(fake).get("up1");
    expect(text(sent?.bytes ?? [])).toBe(FAKE_BACKUP_BYTES);
    expect(host.fileFilters).toEqual([]);

    // The server's Botloft restarts itself; this computer's stays as it is.
    const field = await within(dialog).findByLabelText("The copy's passphrase");
    fireEvent.change(field, { target: { value: FAKE_PASSPHRASE } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Open" }));
    fireEvent.click(await within(dialog).findByRole("button", { name: "Restore and restart" }));
    await waitFor(() => expect(fake.restarts).toBe(1));
    expect(host.installs).toEqual([]);
  });

  test("a new crew's folder is a path on the server", async () => {
    const { fake } = onServer();
    renderApp(fake, new FakeHost());
    await crewOpened("Ops");
    fireEvent.click(screen.getByRole("button", { name: /^New crew/ }));
    const field = await screen.findByLabelText("Folder on the server");
    fireEvent.change(field, { target: { value: "/srv/site" } });
    expect((field as HTMLInputElement).value).toBe("/srv/site");
    expect(screen.queryByRole("button", { name: "Choose folder…" })).toBeNull();
  });
});
