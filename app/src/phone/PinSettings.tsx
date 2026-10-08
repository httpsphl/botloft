// The PIN in "This phone" (spec 28.13): why it is worth having, and turning
// it on and off. The owner decides; nothing here insists.

import { useEffect, useState } from "react";
import { useT } from "../i18n";
import { Callout } from "../ui/Callout";
import { PhoneButton } from "./parts";
import { validPin } from "./vault";

/** What the page can do about its lock. */
export interface LockApi {
  /** `old`: a session from before the PIN, which needs the phone connected again. */
  state(): Promise<"off" | "on" | "old">;
  setPin(pin: string): Promise<void>;
  /** False if the PIN was wrong. */
  removePin(pin: string): Promise<boolean>;
  /** Whether the owner said "not now" to the notice on the inbox. */
  nudgeDismissed(): boolean;
  dismissNudge(): void;
}

const digits = (value: string) => value.replace(/\D/g, "");

function PinField({
  label,
  value,
  change,
}: {
  label: string;
  value: string;
  change(v: string): void;
}) {
  return (
    <label className="flex flex-col gap-1 text-muted text-sm">
      {label}
      <input
        value={value}
        type="password"
        inputMode="numeric"
        autoComplete="off"
        maxLength={10}
        onChange={(event) => change(digits(event.target.value))}
        className="h-12 rounded-xl border border-line-strong bg-canvas px-3 font-mono text-base text-ink tracking-[0.2em]"
      />
    </label>
  );
}

export function PinSettings({ lock }: { lock: LockApi }) {
  const t = useT().phone.pin;
  const [state, setState] = useState<"off" | "on" | "old" | null>(null);
  const [form, setForm] = useState<"on" | "off" | null>(null);
  const [pin, setPin] = useState("");
  const [again, setAgain] = useState("");
  const [problem, setProblem] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    void lock.state().then((now) => live && setState(now));
    return () => {
      live = false;
    };
  }, [lock]);

  const close = () => {
    setForm(null);
    setPin("");
    setAgain("");
    setProblem(null);
  };

  const save = async () => {
    if (!validPin(pin)) {
      setProblem(t.invalid);
      return;
    }
    if (pin !== again) {
      setProblem(t.mismatch);
      return;
    }
    setBusy(true);
    await lock.setPin(pin);
    setBusy(false);
    setState("on");
    close();
  };

  const turnOff = async () => {
    setBusy(true);
    const done = await lock.removePin(pin);
    setBusy(false);
    if (done) {
      setState("off");
      close();
    } else {
      setPin("");
      setProblem(t.wrongPin);
    }
  };

  if (state === null) {
    return null;
  }
  return (
    <section className="flex flex-col gap-3">
      <h2 className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">{t.title}</h2>
      <p className="text-ink-soft text-sm leading-relaxed">{t.why}</p>
      {state === "old" && <Callout tone="info" title={t.old} />}
      {state !== "old" && form === null && (
        <>
          <p className="text-sm">{state === "on" ? t.on : t.off}</p>
          <PhoneButton
            look={state === "on" ? "quiet" : "primary"}
            onClick={() => setForm(state === "on" ? "off" : "on")}
            className="flex-none"
          >
            {state === "on" ? t.turnOff : t.turnOn}
          </PhoneButton>
        </>
      )}
      {form === "on" && (
        <div className="flex flex-col gap-3">
          <PinField label={t.newPin} value={pin} change={setPin} />
          <PinField label={t.again} value={again} change={setAgain} />
          {problem && (
            <p role="alert" className="text-danger text-sm">
              {problem}
            </p>
          )}
          <div className="flex gap-3">
            <PhoneButton look="primary" disabled={busy || pin === ""} onClick={save}>
              {t.save}
            </PhoneButton>
            <PhoneButton onClick={close}>{t.cancel}</PhoneButton>
          </div>
        </div>
      )}
      {form === "off" && (
        <div className="flex flex-col gap-3">
          <PinField label={t.current} value={pin} change={setPin} />
          {problem && (
            <p role="alert" className="text-danger text-sm">
              {problem}
            </p>
          )}
          <div className="flex gap-3">
            <PhoneButton look="danger" disabled={busy || pin === ""} onClick={turnOff}>
              {t.turnOff}
            </PhoneButton>
            <PhoneButton onClick={close}>{t.cancel}</PhoneButton>
          </div>
        </div>
      )}
    </section>
  );
}

/** On the inbox while there is no PIN: the reason, and a way to say not now. */
export function PinNudge({ lock, open }: { lock: LockApi; open(): void }) {
  const t = useT().phone.pin;
  const [show, setShow] = useState(false);

  useEffect(() => {
    let live = true;
    void lock.state().then((now) => live && setShow(now === "off" && !lock.nudgeDismissed()));
    return () => {
      live = false;
    };
  }, [lock]);

  if (!show) {
    return null;
  }
  return (
    <Callout tone="info" title={t.nudgeTitle}>
      <p className="mb-2">{t.why}</p>
      <div className="flex gap-3">
        <PhoneButton look="primary" onClick={open}>
          {t.nudgeYes}
        </PhoneButton>
        <PhoneButton
          onClick={() => {
            lock.dismissNudge();
            setShow(false);
          }}
        >
          {t.nudgeNo}
        </PhoneButton>
      </div>
    </Callout>
  );
}
