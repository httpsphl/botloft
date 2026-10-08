// The phone page when it is locked (spec 28.13): the PIN, how many tries are
// left, the wait after a wrong one, and the way out for whoever forgot it.

import { useEffect, useState } from "react";
import { useT } from "../i18n";
import { Callout } from "../ui/Callout";
import { Confirm } from "../ui/Confirm";
import { PhoneButton } from "./parts";
import type { Session } from "./store";
import type { Unlock } from "./vault";

type Said = { wrong: number } | { wiped: true } | null;

export function LockScreen({
  unlock,
  forget,
  onOpen,
  onGone,
}: {
  unlock(pin: string): Promise<Unlock>;
  /** Forgets the session on this phone: the PIN was forgotten. */
  forget(): Promise<void>;
  onOpen(session: Session): void;
  /** The session is gone (too many wrong PINs, or forgotten). */
  onGone(): void;
}) {
  const t = useT().phone.lock;
  const [pin, setPin] = useState("");
  const [busy, setBusy] = useState(false);
  const [said, setSaid] = useState<Said>(null);
  const [wait, setWait] = useState(0);
  const [asking, setAsking] = useState(false);

  // The wait counts down on screen.
  useEffect(() => {
    if (wait <= 0) {
      return;
    }
    const timer = setInterval(() => setWait((left) => Math.max(0, left - 1000)), 1000);
    return () => clearInterval(timer);
  }, [wait]);

  const open = async () => {
    setBusy(true);
    const result = await unlock(pin);
    setBusy(false);
    setPin("");
    if ("ok" in result) {
      onOpen(result.ok);
    } else if ("wiped" in result) {
      setSaid({ wiped: true });
    } else if ("waiting" in result) {
      setWait(result.waiting);
    } else {
      setSaid({ wrong: result.left });
      setWait(result.wait);
    }
  };

  return (
    <main className="mx-auto flex min-h-full w-full max-w-md flex-col gap-4 px-4 pt-[max(2rem,env(safe-area-inset-top))] pb-[max(2rem,env(safe-area-inset-bottom))]">
      <h1 className="font-semibold text-xl">{t.title}</h1>
      {said && "wiped" in said ? (
        <>
          <Callout tone="danger" title={t.wiped} />
          <PhoneButton look="primary" onClick={onGone} className="flex-none">
            OK
          </PhoneButton>
        </>
      ) : (
        <>
          <form
            className="flex flex-col gap-3"
            onSubmit={(event) => {
              event.preventDefault();
              if (pin !== "" && wait <= 0) {
                void open();
              }
            }}
          >
            <label className="flex flex-col gap-1 text-muted text-sm">
              {t.pinLabel}
              <input
                value={pin}
                type="password"
                inputMode="numeric"
                autoComplete="off"
                maxLength={10}
                onChange={(event) => setPin(event.target.value.replace(/\D/g, ""))}
                className="h-14 rounded-xl border border-line-strong bg-canvas px-3 text-center font-mono text-2xl text-ink tracking-[0.3em]"
              />
            </label>
            {said && "wrong" in said && wait <= 0 && (
              <p role="alert" className="text-danger text-sm">
                {t.wrong(said.wrong)}
              </p>
            )}
            {wait > 0 && (
              <p role="alert" className="text-danger text-sm">
                {t.wait(Math.ceil(wait / 1000))}
              </p>
            )}
            <PhoneButton
              look="primary"
              type="submit"
              disabled={busy || pin === "" || wait > 0}
              className="flex-none"
            >
              {busy ? t.unlocking : t.unlock}
            </PhoneButton>
          </form>
          <button
            type="button"
            onClick={() => setAsking(true)}
            className="min-h-11 self-start font-medium text-sm text-work underline underline-offset-2"
          >
            {t.forgot}
          </button>
        </>
      )}
      {asking && (
        <Confirm
          title={t.forgotTitle}
          confirmLabel={t.forgotConfirm}
          onConfirm={async () => {
            await forget();
            onGone();
          }}
          onClose={() => setAsking(false)}
        >
          {t.forgotText}
        </Confirm>
      )}
    </main>
  );
}
