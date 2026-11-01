// @vitest-environment node
// What the phone sends besides words (spec 28.14): photos made smaller, and
// each file sealed with a key of its own before it reaches the server.

import { describe, expect, test, vi } from "vitest";
import vectorsText from "../../../docs/test-vectors/mobile.json?raw";
import { fromB64, sealFile, sha256Hex } from "./crypto";
import { asJpeg, FILE_MAX, type Outgoing, scaledTo, UploadError, upload } from "./outgoing";

const vectors = JSON.parse(vectorsText);
const text = new TextEncoder();

function outgoing(bytes: Uint8Array, name = "receipt.pdf", type = "application/pdf"): Outgoing {
  return { blob: new Blob([new Uint8Array(bytes)]), name, type };
}

/** A server that keeps what it is sent, and answers as told. */
function server(answer: () => Response | Promise<Response>) {
  const sent: { url: string; init: RequestInit }[] = [];
  const fetch = vi.fn(async (url: string | URL | Request, init?: RequestInit) => {
    sent.push({ url: String(url), init: init ?? {} });
    return answer();
  }) as unknown as typeof globalThis.fetch;
  return { sent, server: { origin: "https://cloud.example.org", fetch, token: "tok_phone" } };
}

const created = () =>
  new Response(JSON.stringify({ id: "fil_abc", size: 1, expires: 1 }), { status: 201 });

describe("a file's seal", () => {
  test("is the same bytes the computer makes, and opens only with its key", async () => {
    const key = fromB64(vectors.file.key);
    const sealed = await sealFile(key, text.encode(vectors.file.plain));
    expect(Array.from(sealed)).toEqual(Array.from(fromB64(vectors.file.sealed)));
  });
});

describe("a photo made smaller", () => {
  test("keeps its proportions under 2048 px, and a small one stays as it is", () => {
    expect(scaledTo(4000, 3000, 2048)).toEqual({ width: 2048, height: 1536 });
    expect(scaledTo(3000, 4000, 2048)).toEqual({ width: 1536, height: 2048 });
    expect(scaledTo(800, 600, 2048)).toEqual({ width: 800, height: 600 });
  });

  test("is named as a JPEG", () => {
    expect(asJpeg("IMG_1234.HEIC")).toBe("IMG_1234.jpg");
    expect(asJpeg("board.png")).toBe("board.jpg");
    expect(asJpeg("")).toBe("photo.jpg");
    expect(asJpeg("notes")).toBe("notes.jpg");
  });
});

describe("leaving a file on the server", () => {
  test("sends it sealed with a new key, which comes back for the computer", async () => {
    const { sent, server: on } = server(created);
    const plain = text.encode("total: 42.00");
    const file = await upload(on, outgoing(plain));
    expect(file).toMatchObject({
      id: "fil_abc",
      name: "receipt.pdf",
      mediaType: "application/pdf",
      size: plain.length,
    });

    const [call] = sent;
    expect(call?.url).toBe("https://cloud.example.org/v1/files");
    expect(call?.init.method).toBe("PUT");
    const headers = call?.init.headers as Record<string, string>;
    expect(headers.Authorization).toBe("Bearer tok_phone");
    const body = call?.init.body as Uint8Array;
    expect(headers["x-botloft-sha256"]).toBe(await sha256Hex(body));
    // What went up is not the file, and the key that came back opens it.
    expect(new TextDecoder().decode(body)).not.toContain("42.00");
    const again = await sealFile(fromB64(file.key), plain);
    expect(Array.from(again)).toEqual(Array.from(body));
    // Each file has a key of its own.
    const other = await upload(on, outgoing(plain));
    expect(other.key).not.toBe(file.key);
  });

  test("says why a file did not go", async () => {
    const plain = text.encode("x");
    const reasonOf = async (answer: () => Response | Promise<Response>) => {
      const failure = await upload(server(answer).server, outgoing(plain)).catch((e) => e);
      expect(failure).toBeInstanceOf(UploadError);
      return (failure as UploadError).reason;
    };
    const said = (reason: string, status: number) => () =>
      new Response(JSON.stringify({ reason }), { status });
    expect(await reasonOf(said("too_big", 413))).toBe("too_big");
    expect(await reasonOf(said("queue_full", 429))).toBe("queue_full");
    expect(await reasonOf(said("internal", 500))).toBe("failed");
    expect(
      await reasonOf(() => {
        throw new TypeError("no network");
      }),
    ).toBe("offline");

    const { sent, server: on } = server(created);
    const big = { blob: { size: FILE_MAX + 1 } as Blob, name: "big.mov", type: "video/mp4" };
    const failure = await upload(on, big).catch((e) => e);
    expect((failure as UploadError).reason).toBe("too_big");
    expect(sent).toHaveLength(0);
  });
});
