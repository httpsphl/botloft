import { LoaderCircle, RefreshCw, RotateCw } from "lucide-react";
import type { ReactNode } from "react";
import { type StoreApi, useStore } from "zustand";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import type { Link } from "./link";

// Plain words on purpose: the owner never has to know that a daemon, a
// port or a scheduled task exists. Technical text goes under "Details".

/** Shown until the app is connected to the part that runs the bots. */
export function Onboarding({ link }: { link: StoreApi<Link> }) {
  const current = useStore(link, (state) => state.current);
  const { check, install, restart } = link.getState();
  const tryAgain = (retry: () => Promise<void>) => (
    <Button variant="primary" icon={RefreshCw} onClick={retry}>
      Try again
    </Button>
  );

  let body: ReactNode;
  switch (current.step) {
    case "checking":
      body = <Waiting text="Opening Botloft…" />;
      break;
    case "installing":
      body = (
        <div className="flex flex-col gap-3">
          <Waiting text={INSTALLING[current.action]} />
          {current.action === "install" && (
            <p className="text-ink-soft text-sm leading-relaxed">
              Botloft keeps your bots running in the background, even after you close this window,
              and starts with Windows.
            </p>
          )}
        </div>
      );
      break;
    case "connecting":
      body = (
        <div className="flex flex-col items-start gap-4">
          <Waiting text="Connecting…" />
          <div className="flex gap-2">
            <Button icon={RefreshCw} onClick={check}>
              Try again
            </Button>
            <Button icon={RotateCw} onClick={restart}>
              Restart Botloft in the background
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
        <Problem title="Botloft couldn't start" action={tryAgain(install)}>
          Botloft runs your bots in the background, and that part didn't start.
          <Details>
            {current.error ?? "It is not running."}
            {"\n"}
            {current.home}
          </Details>
        </Problem>
      );
      break;
    case "outdated":
      body = (
        <Problem title="Botloft couldn't finish updating" action={tryAgain(install)}>
          The part that runs your bots in the background is still on the previous version.
          <Details>
            Running version {current.daemonVersion}. {current.error}
          </Details>
        </Problem>
      );
      break;
    case "foreign":
      body = (
        <Problem title="Another program is in the way" action={tryAgain(check)}>
          Botloft needs port {current.port} on this computer, and another program is using it. Close
          that program and try again.
          <Details>
            127.0.0.1:{current.port} answers but is not Botloft. To use another port, set{" "}
            <code>port</code> in Botloft's config.toml.
          </Details>
        </Problem>
      );
      break;
    case "mismatch":
      body = (
        <Problem
          title="This app doesn't match the Botloft that is running"
          action={tryAgain(check)}
        >
          The Botloft running in the background is newer than this app. Install the latest version
          of Botloft.
          <Details>
            Running version {current.daemonVersion}, protocol {current.daemonProtocol}. This app
            speaks protocol {PROTOCOL_VERSION}.
          </Details>
        </Problem>
      );
      break;
    case "refused":
      body = (
        <Problem title="Botloft couldn't connect" action={tryAgain(check)}>
          Try again. If this keeps happening, reinstall Botloft.
          <Details>{current.reason}</Details>
        </Problem>
      );
      break;
    case "error":
      body = (
        <Problem title="Botloft couldn't connect" action={tryAgain(check)}>
          Try again. If this keeps happening, reinstall Botloft.
          <Details>{current.message}</Details>
        </Problem>
      );
      break;
  }

  return (
    <main className="grid flex-1 place-items-center p-8">
      <div className="w-full max-w-lg">
        <h1 className="font-semibold text-2xl tracking-tight">Botloft</h1>
        <p className="mt-1 mb-6 text-muted">Always-on Claude Code crews.</p>
        {body}
      </div>
    </main>
  );
}

const INSTALLING = {
  install: "Getting Botloft ready…",
  update: "Updating Botloft…",
  restart: "Restarting Botloft…",
} as const;

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
