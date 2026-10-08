// The phone's side of the sealing (spec 28.3), with the browser's own
// WebCrypto: ECDH P-256, HKDF-SHA-256, HMAC-SHA-256 and AES-256-GCM. The
// computer does the same in Rust, and `docs/test-vectors/mobile.json` holds
// bytes both sides must agree on (crypto.test.ts reads it).

const text = new TextEncoder();
const VERSION = 1;

/** base64url without padding, as the computer writes it. */
export function toB64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

export function fromB64(value: string): Uint8Array {
  const padded = value.replaceAll("-", "+").replaceAll("_", "/");
  const binary = atob(padded + "=".repeat((4 - (padded.length % 4)) % 4));
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

export class SealError extends Error {}

function concat(...parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let at = 0;
  for (const part of parts) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}

/** WebCrypto wants buffers it can be sure are not shared. */
const own = (bytes: Uint8Array): Uint8Array<ArrayBuffer> => new Uint8Array(bytes);

async function hmac(key: Uint8Array, ...parts: Uint8Array[]): Promise<Uint8Array> {
  const imported = await crypto.subtle.importKey(
    "raw",
    own(key),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return new Uint8Array(await crypto.subtle.sign("HMAC", imported, own(concat(...parts))));
}

async function hkdf(
  ikm: Uint8Array,
  salt: Uint8Array,
  info: string,
  bytes: number,
): Promise<Uint8Array> {
  const imported = await crypto.subtle.importKey("raw", own(ikm), "HKDF", false, ["deriveBits"]);
  const bits = await crypto.subtle.deriveBits(
    { name: "HKDF", hash: "SHA-256", salt: own(salt), info: text.encode(info) },
    imported,
    bytes * 8,
  );
  return new Uint8Array(bits);
}

/** This phone's key for the exchange: the private half never leaves the browser. */
export interface Keypair {
  privateKey: CryptoKey;
  /** The point, uncompressed (65 bytes). */
  publicRaw: Uint8Array;
}

export async function generateKeypair(): Promise<Keypair> {
  const pair = await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, false, [
    "deriveBits",
  ]);
  const publicRaw = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
  return { privateKey: pair.privateKey, publicRaw };
}

/** What the phone proves when it joins: that it read the QR's secret. */
export function joinProof(secret: Uint8Array, id: string, phonePub: Uint8Array) {
  return hmac(secret, text.encode("botloft-pair-1"), text.encode(id), phonePub);
}

/** What the computer proves when it accepts. */
export function acceptProof(
  secret: Uint8Array,
  id: string,
  daemonPub: Uint8Array,
  phonePub: Uint8Array,
) {
  return hmac(secret, text.encode("botloft-pair-1-accept"), text.encode(id), daemonPub, phonePub);
}

/** Six digits both screens show. */
export async function pairCode(
  secret: Uint8Array,
  daemonPub: Uint8Array,
  phonePub: Uint8Array,
): Promise<string> {
  const bytes = await hkdf(concat(daemonPub, phonePub), secret, "botloft-pair-1-code", 3);
  const bits = ((bytes[0] ?? 0) << 12) | ((bytes[1] ?? 0) << 4) | ((bytes[2] ?? 0) >> 4);
  return String(bits % 1_000_000).padStart(6, "0");
}

/** The two keys of a connection, as bytes (for the tests). */
export async function deriveRaw(
  mine: CryptoKey,
  theirPublic: Uint8Array,
  secret: Uint8Array,
): Promise<{ c2p: Uint8Array; p2c: Uint8Array }> {
  const theirs = await crypto.subtle.importKey(
    "raw",
    own(theirPublic),
    { name: "ECDH", namedCurve: "P-256" },
    false,
    [],
  );
  const shared = new Uint8Array(
    await crypto.subtle.deriveBits({ name: "ECDH", public: theirs }, mine, 256),
  );
  const okm = await hkdf(shared, secret, "botloft-mobile-1", 64);
  return { c2p: okm.slice(0, 32), p2c: okm.slice(32) };
}

/** One key per direction; the browser keeps them and cannot export them. */
export interface Keys {
  c2p: CryptoKey;
  p2c: CryptoKey;
}

export async function importKey(raw: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey("raw", own(raw), "AES-GCM", false, ["encrypt", "decrypt"]);
}

export async function deriveKeys(
  mine: CryptoKey,
  theirPublic: Uint8Array,
  secret: Uint8Array,
): Promise<Keys> {
  const raw = await deriveRaw(mine, theirPublic, secret);
  return { c2p: await importKey(raw.c2p), p2c: await importKey(raw.p2c) };
}

export type Direction = "computerToPhone" | "phoneToComputer";

const DIRECTION: Record<Direction, number> = { computerToPhone: 1, phoneToComputer: 2 };

function parts(direction: Direction, device: string, seq: number) {
  const counter = new Uint8Array(8);
  new DataView(counter.buffer).setBigUint64(0, BigInt(seq));
  const aad = concat(Uint8Array.of(VERSION, DIRECTION[direction]), text.encode(device), counter);
  // The nonce is four zero bytes and the counter.
  return { aad, iv: concat(new Uint8Array(4), counter) };
}

/** `{"v":1,"seq":N,"ct":"..."}`, the text the relay carries. */
export async function seal(
  key: CryptoKey,
  direction: Direction,
  device: string,
  seq: number,
  plain: Uint8Array,
): Promise<string> {
  const { aad, iv } = parts(direction, device, seq);
  const ct = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: own(iv), additionalData: own(aad) },
    key,
    own(plain),
  );
  return JSON.stringify({ v: VERSION, seq, ct: toB64(new Uint8Array(ct)) });
}

/** The counter and the plain text of a sealed message; throws if it does not open. */
export async function open(
  key: CryptoKey,
  direction: Direction,
  device: string,
  body: string,
): Promise<{ seq: number; plain: Uint8Array }> {
  let sealed: { v?: unknown; seq?: unknown; ct?: unknown };
  try {
    sealed = JSON.parse(body);
  } catch {
    throw new SealError("not a sealed message");
  }
  if (
    sealed.v !== VERSION ||
    typeof sealed.seq !== "number" ||
    !Number.isSafeInteger(sealed.seq) ||
    sealed.seq < 0 ||
    typeof sealed.ct !== "string"
  ) {
    throw new SealError("not a sealed message");
  }
  const { aad, iv } = parts(direction, device, sealed.seq);
  try {
    const plain = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: own(iv), additionalData: own(aad) },
      key,
      own(fromB64(sealed.ct)),
    );
    return { seq: sealed.seq, plain: new Uint8Array(plain) };
  } catch {
    throw new SealError("the message does not open");
  }
}
