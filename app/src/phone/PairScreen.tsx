// Connecting this phone (spec 28.3, 28.7): the owner scanned the code on the
// computer. Here the phone gets a name, joins, shows six digits to compare
// with the computer's, and waits for the owner to accept there.

import { useState } from "react";
import { useT } from "../i18n";
import { Callout } from "../ui/Callout";
import { type Pair, PairError, type PairFailure } from "./pair";
import { PhoneButton } from "./parts";
import type { Session } from "./store";

/** A name to start from, by the kind of phone this is. */
export function guessName(
  names: { iphone: string; ipad: string; android: string; other: string },
  agent = typeof navigator === "undefined" ? "" : navigator.userAgent,
): string {
  if (/iPhone|iPod/.test(agent)) {
    return names.iphone;
  }
  if (/iPad/.test(agent)) {
    return names.ipad;
  }
  return /Android/.test(agent) ? names.android : names.other;
}

const spaced = (code: string) => `${code.slice(0, 3)} ${code.slice(3)}`;

export function PairScreen({
  pair,
  onSession,
}: {
  /** `null` when the page was not opened from a code. */
  pair: Pair | null;
  onSession(session: Session): void;
}) {
  const t = useT().phone;
  const p = t.pair;
  const [name, setName] = useState(() => guessName(p.names));
  const [step, setStep] = useState<"form" | "joining" | "waiting">("form");
  const [code, setCode] = useState("");
  const [failure, setFailure] = useState<PairFailure | null>(null);

  const connect = async () => {
    if (!pair) {
      return;
    }
    setFailure(null);
    setStep("joining");
    try {
      const joined = await pair.join(name.trim() || guessName(p.names));
      setCode(joined.code);
      setStep("waiting");
      onSession(await joined.collect());
    } catch (error) {
      setFailure(error instanceof PairError ? error.reason : "failed");
      setStep("form");
    }
  };

  return (
    <main className="mx-auto flex min-h-full w-full max-w-md flex-col gap-4 px-4 pt-[max(2rem,env(safe-area-inset-top))] pb-[max(2rem,env(safe-area-inset-bottom))]">
      <h1 className="font-semibold text-xl">{p.title}</h1>
      {!pair ? (
        <p className="text-ink-soft">{p.notALink}</p>
      ) : step === "waiting" ? (
        <>
          <p className="font-mono text-4xl tracking-[0.2em]">{spaced(code)}</p>
          <p className="text-ink-soft">{p.compare(spaced(code))}</p>
          <p role="status" className="text-muted text-sm">
            {p.waiting}
          </p>
        </>
      ) : (
        <>
          <p className="text-ink-soft">{p.intro}</p>
          <label className="flex flex-col gap-1 text-muted text-sm">
            {p.nameLabel}
            <input
              value={name}
              maxLength={60}
              onChange={(event) => setName(event.target.value)}
              className="h-12 rounded-xl border border-line-strong bg-canvas px-3 text-base text-ink"
            />
          </label>
          <PhoneButton
            look="primary"
            disabled={step === "joining"}
            onClick={connect}
            className="flex-none"
          >
            {step === "joining" ? p.connecting : p.connect}
          </PhoneButton>
          {failure && <Callout tone="danger" title={p[failure]} />}
        </>
      )}
    </main>
  );
}
