// The fake daemon's account and cloud copies (spec 27.5), all in memory. A
// sign-in waits until `openFakeCloudLink` plays the owner opening the link in
// the e-mail. Nothing leaves the page.

import type { FakeBotloft, Handlers } from "./fake";
import { type CloudCopy, type CloudStatus, RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type CloudMethods = Extract<keyof Handlers, `cloud.${string}`>;

const URL = "https://cloud.botloft.example";
const QUOTA = 200 * 1024 * 1024;
const COPY_SIZE = 48_000;

interface CloudState {
  signedIn: boolean;
  email: string | null;
  /** The e-mail a waiting sign-in went to. */
  pending: string | null;
  copies: CloudCopy[];
  next: number;
}

const states = new WeakMap<FakeBotloft, CloudState>();

function stateOf(fake: FakeBotloft): CloudState {
  let state = states.get(fake);
  if (!state) {
    state = { signedIn: false, email: null, pending: null, copies: [], next: 1 };
    states.set(fake, state);
  }
  return state;
}

function refused(reason: string, message: string): RpcError {
  return new RpcError(RpcErrorCode.validation, message, reason);
}

function signedIn(state: CloudState): void {
  if (!state.signedIn) throw refused("not_signed_in", "sign in to the account first");
}

/** The owner opens the link in the e-mail: the waiting sign-in finishes. */
export function openFakeCloudLink(fake: FakeBotloft): void {
  const state = stateOf(fake);
  if (!state.pending) return;
  state.signedIn = true;
  state.email = state.pending;
  state.pending = null;
  fake.emit({ name: "cloud.signed_in", params: { email: state.email } });
}

/** The link ran out before the owner opened it. */
export function expireFakeCloudLink(fake: FakeBotloft): void {
  const state = stateOf(fake);
  if (!state.pending) return;
  state.pending = null;
  fake.emit({ name: "cloud.signin_expired", params: null });
}

export function cloudHandlers(fake: FakeBotloft): Pick<Handlers, CloudMethods> {
  const state = stateOf(fake);
  const used = () => state.copies.reduce((sum, copy) => sum + copy.size, 0);
  return {
    "cloud.status": () => {
      const status: CloudStatus = {
        url: URL,
        signedIn: state.signedIn,
        pending: state.pending !== null,
      };
      if (state.signedIn && state.email) {
        status.email = state.email;
        status.used = used();
        status.quota = QUOTA;
      }
      return status;
    },
    "cloud.signin": ({ email }) => {
      if (!email.includes("@")) throw refused("bad_email", "that is not an e-mail address");
      state.pending = email.trim().toLowerCase();
      return { wait: 600 };
    },
    "cloud.signin_cancel": () => {
      state.pending = null;
      return null;
    },
    "cloud.signout": () => {
      state.signedIn = false;
      state.email = null;
      state.pending = null;
      return null;
    },
    "cloud.upload": ({ passphrase }) => {
      signedIn(state);
      if ([...passphrase].length < 8) {
        throw refused("short_passphrase", "the passphrase must have at least 8 characters");
      }
      const copy: CloudCopy = {
        id: `cpy_fake${state.next++}`,
        size: COPY_SIZE,
        created: fake.now,
      };
      // Only the newest five are kept (spec 27.2).
      state.copies = [copy, ...state.copies].slice(0, 5);
      fake.emit({
        name: "cloud.progress",
        params: { direction: "upload", sent: COPY_SIZE, total: COPY_SIZE },
      });
      return copy;
    },
    "cloud.copies": () => {
      signedIn(state);
      return { copies: [...state.copies] };
    },
    "cloud.download": ({ id }) => {
      signedIn(state);
      if (!state.copies.some((copy) => copy.id === id)) {
        throw refused("not_found", "there is no such copy");
      }
      return { path: "C:\\Users\\owner\\AppData\\Local\\Botloft\\cloud-download\\copy.botloft" };
    },
    "cloud.delete": ({ id }) => {
      signedIn(state);
      state.copies = state.copies.filter((copy) => copy.id !== id);
      return null;
    },
    "cloud.delete_account": () => {
      signedIn(state);
      return null;
    },
  };
}
