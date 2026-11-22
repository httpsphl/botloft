// Files in pieces between this app and Botloft on another computer (spec
// 42.7): read from the daemon and written where the owner picks or into the
// app's temporary folder, or read from a file the owner picked and sent.

import type { BotloftApi } from "./api";
import { decodedSize } from "./base64";
import type { Host, PickedFile } from "./host";
import type { FileChunk } from "./protocol.gen";

/** Bytes read from a picked file at a time: the most the daemon takes in one piece. */
const PIECE = 1024 * 1024;

export type ReadPiece = (offset: number) => Promise<FileChunk>;

/** The file's name in a path of either computer, `/` or `\`. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() || "file";
}

/** A bot's file, in pieces (`files.chunk`). */
export function botFile(api: BotloftApi, botId: string, path: string): ReadPiece {
  return (offset) => api.call("files.chunk", { botId, path, offset });
}

/** The backup `backup.export` wrote, in pieces. */
export function exportedBackup(api: BotloftApi, path: string): ReadPiece {
  return (offset) => api.call("backup.chunk", { path, offset });
}

/** Every piece into the download `id`, which ends kept; on a failure, thrown away. */
async function fill(host: Host, id: string, read: ReadPiece, open: boolean): Promise<void> {
  let offset = 0;
  try {
    for (;;) {
      const piece = await read(offset);
      if (piece.data === "") {
        break;
      }
      await host.transfers.write(id, piece.data);
      offset += decodedSize(piece.data);
      if (offset >= piece.size) {
        break;
      }
    }
  } catch (failure) {
    await host.transfers.end(id, { keep: false, open: false }).catch(() => {});
    throw failure;
  }
  await host.transfers.end(id, { keep: true, open });
}

/** Saves a file from the daemon where the owner picks; false when they cancel. */
export async function saveFrom(host: Host, name: string, read: ReadPiece): Promise<boolean> {
  const id = await host.transfers.save(name);
  if (id === null) {
    return false;
  }
  await fill(host, id, read, false);
  return true;
}

/** Opens a file from the daemon with the program the system uses for it. */
export async function openFrom(host: Host, name: string, read: ReadPiece): Promise<void> {
  await fill(host, await host.transfers.temp(name), read, true);
}

/** Sends a picked file to the daemon; where it is there, for what takes a path. */
export async function sendPicked(api: BotloftApi, host: Host, picked: PickedFile): Promise<string> {
  const { uploadId } = await api.call("uploads.start", { name: picked.name, size: picked.size });
  try {
    for (let offset = 0; offset < picked.size; offset += PIECE) {
      const data = await host.transfers.read(picked.id, offset, PIECE);
      await api.call("uploads.chunk", { uploadId, offset, data });
    }
    return (await api.call("uploads.finish", { uploadId })).path;
  } catch (failure) {
    await api.call("uploads.cancel", { uploadId }).catch(() => {});
    throw failure;
  }
}
