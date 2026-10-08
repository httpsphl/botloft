// A code waiting to be scanned (spec 28.3, 28.7): the picture, the time left,
// and, once a phone scanned it, the name and the two codes to compare.

import { useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { MobilePairStarted, MobilePairing as Pending } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { QrCode } from "../../ui/QrCode";
import { backupError } from "./backupError";
import { clock, useSecondsLeft } from "./useMobile";

/** "482913" as "482 913", easier to compare by eye. */
const spaced = (code: string) => `${code.slice(0, 3)} ${code.slice(3)}`;

export function MobilePairing({
  pending,
  started,
  done,
  lapsed,
  refresh,
}: {
  pending: Pending;
  started: MobilePairStarted | null;
  /** The owner is done with this code: refused, cancelled or connected. */
  done(): void;
  /** It ran out. */
  lapsed(): void;
  refresh(): Promise<void>;
}) {
  const m = useT().account.mobile;
  const reasons = useT().account.cloud.reasons;
  const api = useApi();
  const left = useSecondsLeft(pending.expiresAt);
  const [accepting, setAccepting] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  // The time is up: the daemon forgets the code, and this one goes away.
  const over = useRef(false);
  useEffect(() => {
    if (left === 0 && !accepting && !over.current) {
      over.current = true;
      lapsed();
      void refresh();
    }
  }, [left, accepting, lapsed, refresh]);

  const answer = async (accept: boolean) => {
    setProblem(null);
    setAccepting(accept);
    try {
      await api.call("mobile.pair_confirm", { pairId: pending.pairId, accept });
      if (!accept) {
        done();
      }
    } catch (failure) {
      setAccepting(false);
      setProblem(backupError(failure, reasons));
    }
  };

  const cancel = async () => {
    done();
    await api.call("mobile.pair_cancel", { pairId: pending.pairId }).catch(() => null);
    await refresh();
  };

  return (
    <div className="flex flex-col gap-3 rounded-md border border-line p-4">
      {pending.joined ? (
        <>
          <p className="font-medium text-sm">{m.wants(pending.joined.name)}</p>
          <p className="font-mono text-3xl tracking-[0.2em]">{spaced(pending.joined.code)}</p>
          <p className="text-ink-soft text-sm">{m.compare(spaced(pending.joined.code))}</p>
          <div className="flex gap-3">
            <Button disabled={accepting} onClick={() => answer(true)}>
              {accepting ? m.connecting : m.accept}
            </Button>
            <Button variant="ghost" disabled={accepting} onClick={() => answer(false)}>
              {m.refuse}
            </Button>
          </div>
        </>
      ) : (
        <>
          {started && <QrCode text={started.url} label={m.qrLabel} />}
          <p className="text-ink-soft text-sm">{m.scan}</p>
          <p className="text-muted text-xs">{m.expires(clock(left))}</p>
          <Button variant="ghost" className="self-start" onClick={cancel}>
            {m.cancel}
          </Button>
        </>
      )}
      {problem && (
        <Callout tone="danger" title={m.failed}>
          {problem}
        </Callout>
      )}
    </div>
  );
}
