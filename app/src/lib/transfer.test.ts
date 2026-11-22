import { describe, expect, test } from "vitest";
import { encodeBytes } from "./base64";
import { FakeBotloft } from "./fake";
import { FakeHost } from "./fakeHost";
import { fakeUploads } from "./fakeUploads";
import type { FileChunk } from "./protocol.gen";
import { baseName, openFrom, type ReadPiece, saveFrom, sendPicked } from "./transfer";

/** A file read in pieces of `piece` bytes, as the daemon would. */
function pieces(bytes: Uint8Array, piece: number): ReadPiece & { asked: number[] } {
  const asked: number[] = [];
  const read = (offset: number): Promise<FileChunk> => {
    asked.push(offset);
    return Promise.resolve({
      data: encodeBytes(bytes.subarray(offset, offset + piece)),
      size: bytes.length,
    });
  };
  return Object.assign(read, { asked });
}

const bytesOf = (text: string) => new TextEncoder().encode(text);

describe("files in pieces", () => {
  test("a save asks for every piece, in order, and keeps the file", async () => {
    const host = new FakeHost();
    const read = pieces(bytesOf("hello there"), 4);
    expect(await saveFrom(host, "notes.txt", read)).toBe(true);
    expect(read.asked).toEqual([0, 4, 8]);
    const [saved] = host.transfers.downloads;
    expect(saved?.kept).toBe(true);
    expect(saved?.opened).toBe(false);
    expect(new TextDecoder().decode(new Uint8Array(saved?.bytes ?? []))).toBe("hello there");
  });

  test("a cancelled save reads nothing", async () => {
    const host = new FakeHost();
    host.transfers.saveChosen = false;
    const read = pieces(bytesOf("hello"), 4);
    expect(await saveFrom(host, "notes.txt", read)).toBe(false);
    expect(read.asked).toEqual([]);
  });

  test("an empty file is saved too", async () => {
    const host = new FakeHost();
    expect(await saveFrom(host, "empty.txt", pieces(new Uint8Array(), 4))).toBe(true);
    expect(host.transfers.downloads[0]?.kept).toBe(true);
  });

  test("an open goes through the temporary folder", async () => {
    const host = new FakeHost();
    await openFrom(host, "chart.png", pieces(bytesOf("png"), 4));
    expect(host.transfers.downloads[0]).toMatchObject({ temp: true, kept: true, opened: true });
  });

  test("a piece that fails throws the download away", async () => {
    const host = new FakeHost();
    const failing: ReadPiece = () => Promise.reject(new Error("lost the connection"));
    await expect(saveFrom(host, "notes.txt", failing)).rejects.toThrow("lost the connection");
    expect(host.transfers.downloads[0]?.kept).toBe(false);
  });

  test("a picked file goes in pieces and says where it is", async () => {
    const fake = new FakeBotloft();
    const host = new FakeHost();
    const bytes = new Uint8Array(2 * 1024 * 1024 + 3).map((_, at) => at % 251);
    host.transfers.nextPick = { name: "history.zip", bytes };
    const picked = await host.transfers.pick();
    if (!picked) {
      throw new Error("nothing picked");
    }
    const path = await sendPicked(fake, host, picked);
    expect(path).toBe("/var/lib/botloft/home/uploads/up1/history.zip");
    expect(fakeUploads(fake).get("up1")?.bytes).toEqual([...bytes]);
  });

  test("a path's name is its last part on either computer", () => {
    expect(baseName("/var/lib/botloft/home/exports/copy.botloft")).toBe("copy.botloft");
    expect(baseName("C:\\Users\\owner\\exports\\copy.botloft")).toBe("copy.botloft");
    expect(baseName("")).toBe("file");
  });
});
