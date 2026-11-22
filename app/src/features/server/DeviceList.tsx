// The devices that reach these agents with a key of their own (spec 42.6),
// kept current by `devices.changed`, each one removable.

import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { when } from "../../lib/format";
import type { Device } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";

/**
 * `always` on a server or from another computer; elsewhere the list shows
 * only once a device was paired, so a computer that never had one stays
 * plain.
 */
export function DeviceList({ always }: { always: boolean }) {
  const d = useT().server.devices;
  const api = useApi();
  const [devices, setDevices] = useState<Device[] | null>(null);
  const [leaving, setLeaving] = useState<Device | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const connection = api.connection();
  const own = connection.kind === "open" ? connection.deviceId : undefined;

  const refresh = useCallback(async () => {
    try {
      setDevices((await api.call("devices.list")).devices);
    } catch {
      // The list keeps what it had.
    }
  }, [api]);

  useEffect(() => {
    void refresh();
    return api.subscribe((event) => {
      if (event.name === "devices.changed") {
        setDevices(event.params.devices);
      }
    });
  }, [api, refresh]);

  const remove = async (device: Device) => {
    setProblem(null);
    try {
      setDevices((await api.call("devices.revoke", { deviceId: device.id })).devices);
    } catch (failure) {
      setProblem(errorText(failure));
    }
  };

  if (!devices || (devices.length === 0 && !always)) {
    return null;
  }
  return (
    <div className="flex flex-col gap-2">
      <h4 className="font-medium text-sm">{d.title}</h4>
      <p className="text-muted text-xs">{d.hint}</p>
      {devices.length === 0 ? (
        <p className="text-muted text-sm">{d.none}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-line rounded-md border border-line">
          {devices.map((device) => (
            <li key={device.id} className="flex items-center justify-between gap-3 px-3 py-2">
              <div className="min-w-0">
                <p className="truncate font-medium text-sm">
                  {device.id === own ? `${device.name} (${d.thisOne})` : device.name}
                </p>
                <p className="text-muted text-xs">
                  {device.lastSeenAt !== undefined ? d.seen(when(device.lastSeenAt)) : d.never}
                </p>
              </div>
              <Button variant="ghost" onClick={() => setLeaving(device)}>
                {d.remove}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {problem && (
        <Callout tone="danger" title={d.failed}>
          {problem}
        </Callout>
      )}
      {leaving && (
        <Confirm
          title={d.removeTitle(leaving.name)}
          confirmLabel={d.remove}
          onConfirm={() => remove(leaving)}
          onClose={() => setLeaving(null)}
        >
          {d.removeText}
        </Confirm>
      )}
    </div>
  );
}
