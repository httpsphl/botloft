// Asking for the sign-in link and waiting for it (spec 27.3, 27.6).

import { Mail } from "lucide-react";
import { useEffect, useState } from "react";
import { useLocale, useT } from "../../i18n";
import type { CloudStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { TextField } from "../../ui/Field";
import { backupError } from "./backupError";

/** The link can be asked for again after this long (the server holds it a minute). */
const RESEND_AFTER_MS = 60_000;

export function CloudSignIn({
  status,
  refresh,
  expired,
  forgetExpired,
}: {
  status: CloudStatus;
  refresh(): Promise<void>;
  expired: boolean;
  forgetExpired(): void;
}) {
  const t = useT().account.cloud;
  const { locale } = useLocale();
  const api = useApi();
  const [email, setEmail] = useState("");
  const [sentTo, setSentTo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [canResend, setCanResend] = useState(false);

  // A link asked for before this screen opened still waits.
  const waiting = status.pending;

  useEffect(() => {
    if (!waiting) {
      setCanResend(false);
      return;
    }
    const timer = setTimeout(() => setCanResend(true), RESEND_AFTER_MS);
    return () => clearTimeout(timer);
  }, [waiting]);

  const ask = async (address: string) => {
    setBusy(true);
    setProblem(null);
    forgetExpired();
    try {
      await api.call("cloud.signin", { email: address, locale });
      setSentTo(address);
      setCanResend(false);
      await refresh();
    } catch (failure) {
      setProblem(backupError(failure, t.reasons));
    }
    setBusy(false);
  };

  const cancel = async () => {
    await api.call("cloud.signin_cancel").catch(() => null);
    setSentTo(null);
    await refresh();
  };

  return (
    <div className="flex flex-col gap-3">
      <p className="text-ink-soft text-sm leading-relaxed">{t.intro}</p>
      {waiting ? (
        <div className="flex flex-col gap-3">
          <p role="status" className="text-sm">
            {sentTo ? t.linkSent(sentTo) : t.linkWaiting}
          </p>
          <div className="flex gap-2">
            <Button disabled={busy || !canResend || !sentTo} onClick={() => sentTo && ask(sentTo)}>
              {t.resend}
            </Button>
            <Button variant="ghost" onClick={cancel}>
              {t.cancel}
            </Button>
          </div>
        </div>
      ) : (
        <form
          className="flex items-end gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            void ask(email.trim());
          }}
        >
          <div className="min-w-0 flex-1">
            <TextField
              type="email"
              autoComplete="email"
              label={t.email}
              value={email}
              onChange={(event) => setEmail(event.target.value)}
            />
          </div>
          <Button icon={Mail} type="submit" disabled={busy || !email.includes("@")}>
            {busy ? t.sendingLink : t.sendLink}
          </Button>
        </form>
      )}
      {expired && !waiting && <p className="text-warn text-sm">{t.expired}</p>}
      {problem && (
        <Callout tone="danger" title={t.failed}>
          {problem}
        </Callout>
      )}
    </div>
  );
}
