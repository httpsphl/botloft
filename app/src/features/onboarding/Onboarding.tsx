import { LoaderCircle, Play, RefreshCw } from "lucide-react";
import type { ReactNode } from "react";
import { type StoreApi, useStore } from "zustand";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import type { Link } from "./link";

/** Shown until the app is connected to the daemon. */
export function Onboarding({ link }: { link: StoreApi<Link> }) {
  const current = useStore(link, (state) => state.current);
  const { check, start } = link.getState();
  const again = (
    <Button icon={RefreshCw} onClick={check}>
      Check again
    </Button>
  );

  let body: ReactNode;
  switch (current.step) {
    case "checking":
      body = <Waiting text="Looking for the Botloft daemon…" />;
      break;
    case "connecting":
      body = (
        <div className="flex flex-col items-start gap-4">
          <Waiting text="Connecting to the Botloft daemon…" />
          {again}
        </div>
      );
      break;
    case "starting":
      body = <Waiting text="Starting the Botloft daemon…" />;
      break;
    case "connected":
      body = null;
      break;
    case "stopped":
      body = (
        <div className="flex flex-col gap-4">
          <p className="text-ink-soft leading-relaxed">
            The daemon runs your bots and keeps them running after this window closes. It is not
            running yet.
          </p>
          {current.error && (
            <Callout tone="danger" title="The daemon did not start">
              {current.error}
              <span className="mt-1 block font-mono text-xs" data-selectable>
                {current.home}
              </span>
            </Callout>
          )}
          <div className="flex gap-2">
            <Button variant="primary" icon={Play} onClick={start}>
              Start the daemon
            </Button>
            {again}
          </div>
        </div>
      );
      break;
    case "foreign":
      body = (
        <Callout tone="danger" title={`Port ${current.port} is taken`} action={again}>
          Another program listens on 127.0.0.1:{current.port}. Close it, or set another{" "}
          <code className="font-mono">port</code> in the daemon's config.toml.
        </Callout>
      );
      break;
    case "mismatch":
      body = (
        <Callout tone="danger" title="The daemon and the app do not match" action={again}>
          The running daemon ({current.daemonVersion}) speaks protocol {current.daemonProtocol}, and
          this app speaks protocol {PROTOCOL_VERSION}. Install matching versions of both.
        </Callout>
      );
      break;
    case "refused":
      body = (
        <Callout tone="danger" title="The daemon refused this app" action={again}>
          {current.reason}
        </Callout>
      );
      break;
    case "error":
      body = (
        <Callout tone="danger" title="Could not reach the daemon" action={again}>
          {current.message}
        </Callout>
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

function Waiting({ text }: { text: string }) {
  return (
    <p role="status" className="flex items-center gap-2 text-ink-soft">
      <LoaderCircle aria-hidden size={16} className="animate-spin" />
      {text}
    </p>
  );
}
