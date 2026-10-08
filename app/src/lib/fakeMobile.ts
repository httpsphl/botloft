// The fake daemon's phones (spec 28), in memory. A code on screen waits until
// `joinFakePhone` plays a phone scanning it; nothing leaves the page.

import type { FakeBotloft, Handlers } from "./fake";
import { fakeSignedIn } from "./fakeCloud";
import {
  type MobilePairing,
  type MobilePhone,
  type MobileStatus,
  RpcErrorCode,
} from "./protocol.gen";
import { RpcError } from "./rpc";

type MobileMethods = Extract<keyof Handlers, `mobile.${string}`>;

const EXPIRES_IN = 300;

interface MobileState {
  phones: MobilePhone[];
  pending: MobilePairing | null;
  next: number;
}

const states = new WeakMap<FakeBotloft, MobileState>();

function stateOf(fake: FakeBotloft): MobileState {
  let state = states.get(fake);
  if (!state) {
    state = { phones: [], pending: null, next: 1 };
    states.set(fake, state);
  }
  return state;
}

function refused(code: number, reason: string, message: string): RpcError {
  return new RpcError(code, message, reason);
}

function statusOf(fake: FakeBotloft): MobileStatus {
  const state = stateOf(fake);
  const status: MobileStatus = {
    relay: state.phones.length > 0 || state.pending ? "connected" : "off",
    phones: state.phones.map((phone) => ({ ...phone })),
  };
  if (state.pending) status.pending = { ...state.pending };
  return status;
}

function told(fake: FakeBotloft): MobileStatus {
  const status = statusOf(fake);
  fake.emit({ name: "mobile.changed", params: status });
  return status;
}

/** A phone scans the code on screen and shows `code` to be compared. */
export function joinFakePhone(fake: FakeBotloft, name = "Celular da Ana", code = "482913"): void {
  const { pending } = stateOf(fake);
  if (!pending) return;
  pending.joined = { name, code };
  fake.emit({ name: "mobile.pair_request", params: { pairId: pending.pairId, name, code } });
  told(fake);
}

/** Signing out of the account takes every phone with it. */
export function forgetFakePhones(fake: FakeBotloft): void {
  const state = stateOf(fake);
  state.phones = [];
  state.pending = null;
}

export function mobileHandlers(fake: FakeBotloft): Pick<Handlers, MobileMethods> {
  const state = stateOf(fake);
  return {
    "mobile.status": () => statusOf(fake),
    "mobile.pair_start": () => {
      if (!fakeSignedIn(fake)) {
        throw refused(RpcErrorCode.validation, "not_signed_in", "sign in to the account first");
      }
      const pairId = `pair${state.next++}`;
      state.pending = { pairId, expiresAt: fake.now + EXPIRES_IN * 1000 };
      told(fake);
      return {
        pairId,
        url: `https://cloud.botloft.example/m#p=${pairId}&s=secret&d=key`,
        expiresIn: EXPIRES_IN,
      };
    },
    "mobile.pair_cancel": ({ pairId }) => {
      if (state.pending?.pairId === pairId) {
        state.pending = null;
        told(fake);
      }
      return null;
    },
    "mobile.pair_confirm": ({ pairId, accept }) => {
      const pending = state.pending;
      if (!pending || pending.pairId !== pairId || !pending.joined) {
        throw refused(RpcErrorCode.conflict, "conflict", "there is no phone waiting on that code");
      }
      state.pending = null;
      if (accept) {
        state.phones.push({
          id: `dev_phone${state.next++}`,
          name: pending.joined.name,
          pairedAt: fake.now,
          online: true,
        });
      }
      told(fake);
      return null;
    },
    "mobile.revoke": ({ phoneId }) => {
      const before = state.phones.length;
      state.phones = state.phones.filter((phone) => phone.id !== phoneId);
      if (state.phones.length === before) {
        throw refused(RpcErrorCode.notFound, "not_found", `phone ${phoneId}`);
      }
      return told(fake);
    },
  };
}
