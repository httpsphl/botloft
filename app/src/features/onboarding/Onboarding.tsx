import { LoaderCircle, RefreshCw, RotateCw } from "lucide-react";
import type { ReactNode } from "react";
import { type StoreApi, useStore } from "zustand";
import { useT } from "../../i18n";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import type { Link } from "./link";

// Plain words on purpose: the owner never has to know that a daemon, a
// port or a scheduled task exists. Technical text goes under "Details".

/** Shown until the app is connected to the part that runs the bots. */
export function Onboarding({ link }: { link: StoreApi<Link> }) {
  const t = useT();
  const o = t.onboarding;
  const current = useStore(link, (state) => state.current);
  const { check, install, restart } = link.getState();
  const tryAgain = (retry: () => Promise<void>) => (
    <Button variant="primary" icon={RefreshCw} onClick={retry}>
      {t.common.tryAgain}
    </Button>
  );

  let body: ReactNode;
  switch (current.step) {
    case "checking":
      body = <Waiting text={o.opening} />;
      break;
    case "installing":
      body = (
        <div className="flex flex-col gap-3">
          <Waiting text={o.installing[current.action]} />
          {current.action === "install" && (
            <p className="text-ink-soft text-sm leading-relaxed">{o.installNote}</p>
          )}
        </div>
      );
      break;
    case "connecting":
      body = (
        <div className="flex flex-col items-start gap-4">
          <Waiting text={o.connecting} />
          <div className="flex gap-2">
            <Button icon={RefreshCw} onClick={check}>
              {t.common.tryAgain}
            </Button>
            <Button icon={RotateCw} onClick={restart}>
              {o.restartInBackground}
            </Button>
          </div>
        </div>
      );
      break;
    case "connected":
      body = null;
      break;
    case "stopped":
      body = (
        <Problem title={o.stopped.title} action={tryAgain(install)}>
          {o.stopped.body}
          <Details>
            {current.error ?? o.stopped.notRunning}
            {"\n"}
            {current.home}
          </Details>
        </Problem>
      );
      break;
    case "outdated":
      body = (
        <Problem title={o.outdated.title} action={tryAgain(install)}>
          {o.outdated.body}
          <Details>
            {o.outdated.running(current.daemonVersion)} {current.error}
          </Details>
        </Problem>
      );
      break;
    case "foreign":
      body = (
        <Problem title={o.foreign.title} action={tryAgain(check)}>
          {o.foreign.body(current.port)}
          <Details>{o.foreign.detail(current.port)}</Details>
        </Problem>
      );
      break;
    case "mismatch":
      body = (
        <Problem title={o.mismatch.title} action={tryAgain(check)}>
          {o.mismatch.body}
          <Details>
            {o.mismatch.detail(current.daemonVersion, current.daemonProtocol, PROTOCOL_VERSION)}
          </Details>
        </Problem>
      );
      break;
    case "refused":
      body = (
        <Problem title={o.cantConnect.title} action={tryAgain(check)}>
          {o.cantConnect.body}
          <Details>{current.reason}</Details>
        </Problem>
      );
      break;
    case "error":
      body = (
        <Problem title={o.cantConnect.title} action={tryAgain(check)}>
          {o.cantConnect.body}
          <Details>{current.message}</Details>
        </Problem>
      );
      break;
  }

  return (
    <main className="grid flex-1 place-items-center p-8">
      <div className="w-full max-w-lg">
        <h1 className="font-semibold text-2xl tracking-tight">Botloft</h1>
        <p className="mt-1 mb-6 text-muted">{o.tagline}</p>
        {body}
      </div>
    </main>
  );
}

function Problem({
  title,
  action,
  children,
}: {
  title: string;
  action: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col items-start gap-4">
      <div className="w-full">
        <Callout tone="danger" title={title}>
          {children}
        </Callout>
      </div>
      {action}
    </div>
  );
}

function Waiting({ text }: { text: string }) {
  return (
    <p role="status" className="flex items-center gap-2 text-ink-soft">
      <LoaderCircle aria-hidden size={16} className="animate-spin" />
      {text}
    </p>
  );
}
