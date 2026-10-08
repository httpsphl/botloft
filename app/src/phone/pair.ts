// Connecting this phone (spec 28.3): read the QR's address, join the code with
// a proof, show the six digits to compare, and collect the token once the
// owner accepts at the computer.

import {
  acceptProof,
  deriveRaw,
  fromB64,
  generateKeypair,
  importKey,
  joinProof,
  type Keypair,
  pairCode,
  toB64,
} from "./crypto";
import type { Session } from "./store";

/** What the QR holds after the `#`; the browser never sends it to the server. */
export interface Fragment {
  id: string;
  secret: Uint8Array;
  daemonPub: Uint8Array;
}

export function parseFragment(hash: string): Fragment | null {
  const params = new URLSearchParams(hash.replace(/^#/, ""));
  const [id, secret, daemonPub] = [params.get("p"), params.get("s"), params.get("d")];
  if (!id || !secret || !daemonPub) {
    return null;
  }
  try {
    const parsed = { id, secret: fromB64(secret), daemonPub: fromB64(daemonPub) };
    return parsed.secret.length === 32 && parsed.daemonPub.length === 65 ? parsed : null;
  } catch {
    return null;
  }
}

/** Why a connection did not happen; the screen words it. */
export type PairFailure = "expired" | "offline" | "failed";

export class PairError extends Error {
  constructor(readonly reason: PairFailure) {
    super(reason);
  }
}

export interface PairDeps {
  origin: string;
  fetch: typeof fetch;
  sleep(ms: number): Promise<void>;
}

/** A phone that joined the code and waits for the owner. */
export interface Joined {
  /** Six digits, the same as the computer shows. */
  code: string;
  /** Waits until the owner accepts; throws `PairError` if the code ran out. */
  collect(): Promise<Session>;
}

export interface Pair {
  join(name: string): Promise<Joined>;
}

export function pairing(fragment: Fragment, deps: PairDeps): Pair {
  const call = async (path: string, init?: RequestInit) => {
    try {
      return await deps.fetch(`${deps.origin}${path}`, init);
    } catch {
      throw new PairError("offline");
    }
  };
  return {
    async join(name) {
      const keys = await generateKeypair();
      const proof = await joinProof(fragment.secret, fragment.id, keys.publicRaw);
      const joined = await call(`/v1/pairings/${fragment.id}/join`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          phone_pub: toB64(keys.publicRaw),
          device_name: name,
          proof: toB64(proof),
        }),
      });
      if (joined.status === 410) {
        throw new PairError("expired");
      }
      if (!joined.ok) {
        throw new PairError("failed");
      }
      const { poll } = (await joined.json()) as { poll: string };
      const code = await pairCode(fragment.secret, fragment.daemonPub, keys.publicRaw);
      return {
        code,
        async collect() {
          for (;;) {
            const answer = await call(`/v1/pairings/${fragment.id}/result`, {
              headers: { Authorization: `Bearer ${poll}` },
            });
            if (!answer.ok) {
              throw new PairError("failed");
            }
            const result = (await answer.json()) as Record<string, string>;
            if (result.status === "expired") {
              throw new PairError("expired");
            }
            if (result.status === "approved") {
              return finish(result, keys, name);
            }
            await deps.sleep(1000);
          }
        },
      };
    },
  };

  async function finish(
    result: Record<string, string>,
    mine: Keypair,
    name: string,
  ): Promise<Session> {
    const [token, device, peer, daemonPub, proof2] = [
      result.token,
      result.device,
      result.peer,
      result.daemon_pub,
      result.proof2,
    ];
    if (!token || !device || !peer || !daemonPub || !proof2) {
      throw new PairError("failed");
    }
    const theirs = fromB64(daemonPub);
    // The key must be the one in the QR, and the computer must prove it
    // accepted with the QR's secret: a server in the middle can do neither.
    const expected = await acceptProof(fragment.secret, fragment.id, theirs, mine.publicRaw);
    if (!sameBytes(theirs, fragment.daemonPub) || !sameBytes(expected, fromB64(proof2))) {
      throw new PairError("failed");
    }
    // The bytes are kept too: that is what can be stored under a PIN (28.13).
    const raw = await deriveRaw(mine.privateKey, theirs, fragment.secret);
    return {
      token,
      device,
      peer,
      name,
      keys: { c2p: await importKey(raw.c2p), p2c: await importKey(raw.p2c) },
      raw,
      sent: 0,
      received: 0,
    };
  }
}

function sameBytes(a: Uint8Array, b: Uint8Array): boolean {
  return a.length === b.length && a.every((byte, index) => byte === b[index]);
}
