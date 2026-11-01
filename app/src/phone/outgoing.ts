// What the owner sends from the phone besides words (spec 28.14): a photo,
// made smaller so the agent sees it at once, or any file as it is. Each is
// sealed here with a key of its own and left on the server; the key goes to
// the computer inside the sealed message, so the server cannot open it.

import type { PhoneFile } from "../lib/protocol.gen";
import { sealFile, sha256Hex, toB64 } from "./crypto";

/** The biggest file, before the seal (the computer's PHONE_FILE_MAX). */
export const FILE_MAX = 20 * 1024 * 1024;
/** As many files as one message takes (spec 9.5). */
export const FILES_MAX = 10;
/** A photo's longer side, at most. */
const PHOTO_EDGE = 2048;
const THUMB_EDGE = 160;

/** A file the owner picked, still on this phone. */
export interface Outgoing {
  /** What goes: the photo made smaller, or the file as it is. */
  blob: Blob;
  name: string;
  type: string;
  /** A small picture of a photo, as a `data:` address, for the screen. */
  thumb?: string;
}

/** Why a file did not go, in the words the screen knows. */
export class UploadError extends Error {
  constructor(readonly reason: string) {
    super(reason);
  }
}

/** The longer side brought down to `edge`, the proportions kept. */
export function scaledTo(width: number, height: number, edge: number) {
  const scale = Math.min(1, edge / Math.max(width, height, 1));
  return {
    width: Math.max(1, Math.round(width * scale)),
    height: Math.max(1, Math.round(height * scale)),
  };
}

/** `IMG_1234.HEIC` becomes `IMG_1234.jpg`. */
export function asJpeg(name: string): string {
  const base = name.replace(/\.[^./\\]*$/, "").trim();
  return `${base || "photo"}.jpg`;
}

export function asIs(file: File): Outgoing {
  return { blob: file, name: file.name || "file", type: file.type || "application/octet-stream" };
}

function canvasOf(image: CanvasImageSource, width: number, height: number) {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  canvas.getContext("2d")?.drawImage(image, 0, 0, width, height);
  return canvas;
}

function jpeg(canvas: HTMLCanvasElement, quality: number): Promise<Blob | null> {
  return new Promise((resolve) => canvas.toBlob(resolve, "image/jpeg", quality));
}

/**
 * A photo made smaller: at most 2048 px on its longer side, as JPEG. One
 * already small and JPEG goes as it is; one this browser cannot read goes
 * as it is too.
 */
export async function photo(file: File): Promise<Outgoing> {
  if (typeof createImageBitmap !== "function" || typeof document === "undefined") {
    return asIs(file);
  }
  try {
    const bitmap = await createImageBitmap(file, { imageOrientation: "from-image" });
    const small = scaledTo(bitmap.width, bitmap.height, THUMB_EDGE);
    const thumb = canvasOf(bitmap, small.width, small.height).toDataURL("image/jpeg", 0.7);
    const size = scaledTo(bitmap.width, bitmap.height, PHOTO_EDGE);
    if (file.type === "image/jpeg" && size.width === bitmap.width) {
      bitmap.close();
      return { ...asIs(file), thumb };
    }
    const blob = await jpeg(canvasOf(bitmap, size.width, size.height), 0.85);
    bitmap.close();
    if (!blob) {
      return asIs(file);
    }
    return { blob, name: asJpeg(file.name), type: "image/jpeg", thumb };
  } catch {
    return asIs(file);
  }
}

/** Where the phone leaves its files: its server, with its token. */
export interface FileServer {
  origin: string;
  fetch: typeof fetch;
  token: string;
}

/** Seals the file with a new key and leaves it on the server for the computer. */
export async function upload(server: FileServer, file: Outgoing): Promise<PhoneFile> {
  if (file.blob.size > FILE_MAX) {
    throw new UploadError("too_big");
  }
  const key = crypto.getRandomValues(new Uint8Array(32));
  const plain = new Uint8Array(await file.blob.arrayBuffer());
  const sealed = await sealFile(key, plain);
  let answer: Response;
  try {
    answer = await server.fetch(`${server.origin}/v1/files`, {
      method: "PUT",
      headers: {
        Authorization: `Bearer ${server.token}`,
        "Content-Type": "application/octet-stream",
        "x-botloft-sha256": await sha256Hex(sealed),
      },
      body: sealed,
    });
  } catch {
    throw new UploadError("offline");
  }
  if (!answer.ok) {
    const said = ((await answer.json().catch(() => ({}))) as { reason?: string }).reason;
    const known = ["too_big", "queue_full", "rate_limited"];
    throw new UploadError(said && known.includes(said) ? said : "failed");
  }
  const { id } = (await answer.json()) as { id: string };
  return {
    id,
    key: toB64(key),
    name: file.name,
    mediaType: file.type,
    size: plain.length,
  };
}
