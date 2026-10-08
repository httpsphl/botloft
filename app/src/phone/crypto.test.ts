// @vitest-environment node
// The phone's sealing against the bytes the computer's Rust code made
// (docs/test-vectors/mobile.json, written by botloftd's tests).

import { describe, expect, test } from "vitest";
import vectorsText from "../../../docs/test-vectors/mobile.json?raw";
import {
  acceptProof,
  deriveRaw,
  fromB64,
  generateKeypair,
  importKey,
  joinProof,
  open,
  pairCode,
  SealError,
  seal,
  toB64,
} from "./crypto";

const vectors = JSON.parse(vectorsText);
const pair = vectors.pair;
const hex = (value: string) =>
  Uint8Array.from(value.match(/../g) ?? [], (byte) => Number.parseInt(byte, 16));
const same = (a: Uint8Array, b: Uint8Array) => expect(Array.from(a)).toEqual(Array.from(b));

/** The vectors' private key, from the bytes the Rust side used. */
async function privateKey(privateHex: string, publicRaw: Uint8Array) {
  return crypto.subtle.importKey(
    "jwk",
    {
      kty: "EC",
      crv: "P-256",
      d: toB64(hex(privateHex)),
      x: toB64(publicRaw.slice(1, 33)),
      y: toB64(publicRaw.slice(33, 65)),
    },
    { name: "ECDH", namedCurve: "P-256" },
    false,
    ["deriveBits"],
  );
}

describe("the computer's vectors", () => {
  const secret = fromB64(pair.secret);
  const daemonPub = fromB64(pair.daemonPublic);
  const phonePub = fromB64(pair.phonePublic);

  test("the proofs and the code come out the same", async () => {
    same(await joinProof(secret, pair.id, phonePub), fromB64(pair.joinProof));
    same(await acceptProof(secret, pair.id, daemonPub, phonePub), fromB64(pair.acceptProof));
    expect(await pairCode(secret, daemonPub, phonePub)).toBe(pair.code);
  });

  test("the two keys come out the same from the phone's side", async () => {
    const mine = await privateKey(pair.phonePrivateHex, phonePub);
    const keys = await deriveRaw(mine, daemonPub, secret);
    same(keys.c2p, hex(pair.computerToPhoneKeyHex));
    same(keys.p2c, hex(pair.phoneToComputerKeyHex));
  });

  test("what the computer sealed opens, and what the phone seals is the same bytes", async () => {
    for (const vector of vectors.seal) {
      const key = await importKey(hex(vector.keyHex));
      const opened = await open(key, vector.direction, vector.device, vector.body);
      expect(opened.seq).toBe(vector.seq);
      expect(new TextDecoder().decode(opened.plain)).toBe(vector.plain);
      const again = await seal(
        key,
        vector.direction,
        vector.device,
        vector.seq,
        new TextEncoder().encode(vector.plain),
      );
      expect(again).toBe(vector.body);
    }
  });
});

describe("a message that was touched", () => {
  const vector = vectors.seal[0];

  test("does not open with another key, direction, device or counter, or when cut", async () => {
    const key = await importKey(hex(vector.keyHex));
    const other = await importKey(hex(vectors.seal[1].keyHex));
    const body: string = vector.body;
    await expect(open(other, vector.direction, vector.device, body)).rejects.toBeInstanceOf(
      SealError,
    );
    await expect(open(key, "phoneToComputer", vector.device, body)).rejects.toBeInstanceOf(
      SealError,
    );
    await expect(open(key, vector.direction, "dev_other", body)).rejects.toBeInstanceOf(SealError);
    await expect(
      open(key, vector.direction, vector.device, body.replace('"seq":1', '"seq":2')),
    ).rejects.toBeInstanceOf(SealError);
    for (const bad of ["", "nope", '{"v":2,"seq":1,"ct":"AA"}', '{"v":1,"seq":-1,"ct":"AA"}']) {
      await expect(open(key, vector.direction, vector.device, bad)).rejects.toBeInstanceOf(
        SealError,
      );
    }
  });
});

describe("a fresh exchange", () => {
  test("two keypairs agree, and the QR's secret is part of the keys", async () => {
    const [a, b] = [await generateKeypair(), await generateKeypair()];
    expect(a.publicRaw.length).toBe(65);
    expect(a.publicRaw[0]).toBe(4);
    const secret = crypto.getRandomValues(new Uint8Array(32));
    const fromA = await deriveRaw(a.privateKey, b.publicRaw, secret);
    const fromB = await deriveRaw(b.privateKey, a.publicRaw, secret);
    same(fromA.c2p, fromB.c2p);
    same(fromA.p2c, fromB.p2c);
    const other = await deriveRaw(
      a.privateKey,
      b.publicRaw,
      crypto.getRandomValues(new Uint8Array(32)),
    );
    expect(Array.from(other.c2p)).not.toEqual(Array.from(fromA.c2p));
  });

  test("base64url round trips and has no padding", () => {
    for (const length of [0, 1, 2, 3, 31, 32, 65]) {
      const bytes = crypto.getRandomValues(new Uint8Array(length));
      const text = toB64(bytes);
      expect(text).toMatch(/^[A-Za-z0-9_-]*$/);
      same(fromB64(text), bytes);
    }
  });
});
