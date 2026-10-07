// The fake daemon's automatic backup (spec 27.10), in memory. The page never
// sends anything: `setFakeAutoBackup` plays what the real loop would report.

import type { FakeBotloft, Handlers } from "./fake";
import { type AutoBackupEvery, type AutoBackupStatus, RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type AutoMethods = Extract<keyof Handlers, `autobackup.${string}`>;

const DAY = 24 * 3600 * 1000;

const states = new WeakMap<FakeBotloft, AutoBackupStatus>();

function stateOf(fake: FakeBotloft): AutoBackupStatus {
  let state = states.get(fake);
  if (!state) {
    state = { available: true, enabled: false, every: "daily" };
    states.set(fake, state);
  }
  return state;
}

function told(fake: FakeBotloft): AutoBackupStatus {
  const status = { ...stateOf(fake) };
  fake.emit({ name: "autobackup.changed", params: status });
  return status;
}

/** Plays a change the loop makes, such as a copy sent or a failed try. */
export function setFakeAutoBackup(fake: FakeBotloft, patch: Partial<AutoBackupStatus>): void {
  Object.assign(stateOf(fake), patch);
  told(fake);
}

/** Signing out of the account turns it off. */
export function turnOffFakeAutoBackup(fake: FakeBotloft): void {
  const state = stateOf(fake);
  state.enabled = false;
  delete state.nextAt;
}

export function autoBackupHandlers(fake: FakeBotloft): Pick<Handlers, AutoMethods> {
  const state = stateOf(fake);
  const every = (value: AutoBackupEvery) => {
    state.every = value;
  };
  return {
    "autobackup.status": () => ({ ...state }),
    "autobackup.enable": ({ passphrase, every: how }) => {
      if (!state.available) {
        throw new RpcError(
          RpcErrorCode.validation,
          "no safe place for the passphrase",
          "no_keystore",
        );
      }
      if ([...passphrase].length < 8) {
        throw new RpcError(
          RpcErrorCode.validation,
          "the passphrase must have at least 8 characters",
          "short_passphrase",
        );
      }
      every(how);
      state.enabled = true;
      state.nextAt = fake.now;
      delete state.lastError;
      return told(fake);
    },
    "autobackup.set_every": ({ every: how }) => {
      every(how);
      return told(fake);
    },
    "autobackup.disable": () => {
      turnOffFakeAutoBackup(fake);
      return told(fake);
    },
  };
}

/** What a copy sent just now looks like to the screen. */
export function fakeAutoBackupSent(fake: FakeBotloft): void {
  const state = stateOf(fake);
  setFakeAutoBackup(fake, {
    lastOkAt: fake.now,
    nextAt: fake.now + (state.every === "weekly" ? 7 * DAY : DAY),
  });
}
