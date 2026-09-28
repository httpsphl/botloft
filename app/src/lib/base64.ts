// Terminal bytes travel as base64 (spec 8).

const CHUNK = 0x8000;

export function encodeBytes(bytes: Uint8Array): string {
  let binary = "";
  for (let start = 0; start < bytes.length; start += CHUNK) {
    binary += String.fromCharCode(...bytes.subarray(start, start + CHUNK));
  }
  return btoa(binary);
}

/** Text typed or pasted into a terminal, as UTF-8. */
export function encodeText(text: string): string {
  return encodeBytes(new TextEncoder().encode(text));
}

/** xterm's binary strings: one char per byte. */
export function encodeBinary(binary: string): string {
  return btoa(binary);
}

export function decodeBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
