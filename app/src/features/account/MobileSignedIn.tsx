// The phone block once the computer is in the account (spec 28.7): the phones
// that are connected, and the way to connect another.

import { useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { MobilePairStarted, MobilePhone, MobileStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";
import { backupError } from "./backupError";
import { MobilePairing } from "./MobilePairing";

export function MobileSignedIn({
  status,
  refresh,
}: {
  status: MobileStatus;
  refresh(): Promise<void>;
}) {
  const m = useT().account.mobile;
  const reasons = useT().account.cloud.reasons;
  const api = useApi();
  const [started, setStarted] = useState<MobilePairStarted | null>(null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [lapsed, setLapsed] = useState(false);
  const [leaving, setLeaving] = useState<MobilePhone | null>(null);

  const connect = async () => {
    setBusy(true);
    setProblem(null);
    setLapsed(false);
    try {
      setStarted(await api.call("mobile.pair_start"));
    } catch (failure) {
      setProblem(backupError(failure, reasons));
    }
    setBusy(false);
  };

  const disconnect = async (phone: MobilePhone) => {
    try {
      await api.call("mobile.revoke", { phoneId: phone.id });
    } catch (failure) {
      setProblem(backupError(failure, reasons));
    }
    await refresh();
  };

  // A code on screen: this window's own, or one left from before it opened.
  const pending = status.pending;
  const showing = pending && (!started || started.pairId === pending.pairId);

  return (
    <div className="flex flex-col gap-4">
      {status.phones.length === 0 ? (
        <p className="text-muted text-sm">{m.none}</p>
      ) : (
        <div className="flex flex-col gap-2">
          <h4 className="font-medium text-sm">{m.phones}</h4>
          <ul className="flex flex-col divide-y divide-line rounded-md border border-line">
            {status.phones.map((phone) => (
              <li key={phone.id} className="flex items-center justify-between gap-3 px-3 py-2">
                <div className="min-w-0">
                  <p className="truncate font-medium text-sm">{phone.name}</p>
                  <p className={phone.online ? "text-ok text-xs" : "text-muted text-xs"}>
                    {phone.online
                      ? m.online
                      : phone.lastSeenAt !== undefined
                        ? m.seen(when(phone.lastSeenAt))
                        : m.neverSeen}
                  </p>
                </div>
                <Button variant="ghost" onClick={() => setLeaving(phone)}>
                  {m.disconnect}
                </Button>
              </li>
            ))}
          </ul>
          <p className="text-muted text-xs">{m.warning}</p>
        </div>
      )}

      {showing && pending ? (
        <MobilePairing
          pending={pending}
          started={started}
          done={() => setStarted(null)}
          lapsed={() => {
            setStarted(null);
            setLapsed(true);
          }}
          refresh={refresh}
        />
      ) : (
        <div className="flex flex-col gap-2">
          {lapsed && (
            <p role="status" className="text-muted text-sm">
              {m.expired}
            </p>
          )}
          <Button className="self-start" disabled={busy} onClick={connect}>
            {busy ? m.starting : m.connect}
          </Button>
        </div>
      )}

      {problem && (
        <Callout tone="danger" title={m.failed}>
          {problem}
        </Callout>
      )}
      {leaving && (
        <Confirm
          title={m.disconnectTitle(leaving.name)}
          confirmLabel={m.disconnect}
          onConfirm={() => disconnect(leaving)}
          onClose={() => setLeaving(null)}
        >
          {m.disconnectText}
        </Confirm>
      )}
    </div>
  );
}
