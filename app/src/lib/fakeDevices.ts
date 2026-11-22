// The fake daemon's devices with a key of their own (spec 42.6), in memory.
// `pairFakeDevice` plays `botloftd pair` on the computer where it runs.

import type { FakeBotloft, Handlers } from "./fake";
import { type Device, type DeviceList, RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type DeviceMethods = Extract<keyof Handlers, `devices.${string}`>;

interface DevicesState {
  devices: Device[];
  next: number;
}

const states = new WeakMap<FakeBotloft, DevicesState>();

function stateOf(fake: FakeBotloft): DevicesState {
  let state = states.get(fake);
  if (!state) {
    state = { devices: [], next: 1 };
    states.set(fake, state);
  }
  return state;
}

function listOf(fake: FakeBotloft): DeviceList {
  return { devices: stateOf(fake).devices.map((device) => ({ ...device })) };
}

function told(fake: FakeBotloft): DeviceList {
  const list = listOf(fake);
  fake.emit({ name: "devices.changed", params: list });
  return list;
}

/** A device paired where the daemon runs; returns it, newest first in the list. */
export function pairFakeDevice(fake: FakeBotloft, name: string, lastSeenAt?: number): Device {
  const state = stateOf(fake);
  const device: Device = { id: `dev_${state.next++}`, name, pairedAt: fake.now };
  if (lastSeenAt !== undefined) {
    device.lastSeenAt = lastSeenAt;
  }
  state.devices.unshift(device);
  told(fake);
  return device;
}

export function deviceHandlers(fake: FakeBotloft): Pick<Handlers, DeviceMethods> {
  return {
    "devices.list": () => listOf(fake),
    "devices.pair": ({ name }) => {
      const device = pairFakeDevice(fake, name);
      return { device: { ...device }, token: `token-${device.id}` };
    },
    "devices.revoke": ({ deviceId }) => {
      const state = stateOf(fake);
      const before = state.devices.length;
      state.devices = state.devices.filter((device) => device.id !== deviceId);
      if (state.devices.length === before) {
        throw new RpcError(RpcErrorCode.notFound, `device ${deviceId}`, "not_found");
      }
      return told(fake);
    },
  };
}
