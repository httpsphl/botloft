// The setup window (spec 15.7): the mascot, the name and one button. The
// NSIS installer does the work behind it, quietly.

import { X } from "lucide-react";
import { type ReactNode, useCallback, useEffect, useState } from "react";
import { BOTLOFT_COLOR, BotAvatar, type Mood } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import { useThemeRoot } from "../shell/theme";
import { Button } from "../ui/Button";
import { Details } from "../ui/Details";
import { asFailure, type InstallFailure, type SetupHost, type SetupState } from "./host";

type Phase =
  | { kind: "loading" }
  | { kind: "ready"; state: SetupState }
  | { kind: "installing"; state: SetupState }
  | { kind: "done"; state: SetupState; openFailed: boolean }
  | { kind: "failed"; state: SetupState | null; failure: InstallFailure };

/** How long "All set." shows before the app opens, in ms. */
const OPEN_AFTER = 900;

export function Setup({ host, openAfter = OPEN_AFTER }: { host: SetupHost; openAfter?: number }) {
  useThemeRoot();
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });

  useEffect(() => {
    let alive = true;
    host.state().then(
      (state) => alive && setPhase({ kind: "ready", state }),
      (error) => alive && setPhase({ kind: "failed", state: null, failure: asFailure(error) }),
    );
    return () => {
      alive = false;
    };
  }, [host]);

  // The window stays hidden until there is something to show.
  const drawn = phase.kind !== "loading";
  useEffect(() => {
    if (drawn) {
      host.show();
    }
  }, [drawn, host]);

  const install = useCallback(
    (state: SetupState) => {
      setPhase({ kind: "installing", state });
      host.install().then(
        () => setPhase({ kind: "done", state, openFailed: false }),
        (error) => setPhase({ kind: "failed", state, failure: asFailure(error) }),
      );
    },
    [host],
  );

  const open = useCallback(
    (state: SetupState) => {
      host.openApp().catch(() => setPhase({ kind: "done", state, openFailed: true }));
    },
    [host],
  );

  // Done: the app opens by itself after a moment.
  useEffect(() => {
    if (phase.kind !== "done" || phase.openFailed) {
      return;
    }
    const timer = setTimeout(() => open(phase.state), openAfter);
    return () => clearTimeout(timer);
  }, [phase, open, openAfter]);

  const closable = phase.kind !== "installing";
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-canvas text-ink">
      <header data-tauri-drag-region className="flex h-9 shrink-0 items-center justify-end">
        {closable && <CloseButton onClose={() => host.close()} />}
      </header>
      <main className="flex min-h-0 flex-1 flex-col items-center overflow-y-auto px-10 pb-4 text-center">
        {phase.kind === "loading" ? null : (
          <Page phase={phase} onInstall={install} onOpen={open} host={host} />
        )}
      </main>
    </div>
  );
}

function CloseButton({ onClose }: { onClose: () => void }) {
  const t = useT();
  return (
    <button
      type="button"
      aria-label={t.shell.window.close}
      onClick={onClose}
      className="grid h-9 w-11 place-items-center text-ink-soft hover:bg-close hover:text-white"
    >
      <X aria-hidden size={16} strokeWidth={1.5} />
    </button>
  );
}

function Page({
  phase,
  host,
  onInstall,
  onOpen,
}: {
  phase: Exclude<Phase, { kind: "loading" }>;
  host: SetupHost;
  onInstall: (state: SetupState) => void;
  onOpen: (state: SetupState) => void;
}) {
  const s = useT().setup;
  switch (phase.kind) {
    case "ready":
      return <Ready state={phase.state} onInstall={onInstall} onOpen={onOpen} />;
    case "installing":
      return (
        <Frame mood="working" title={s.installing} line={s.installingLine}>
          <div role="progressbar" aria-label={s.progress} className="setup-progress mt-6" />
        </Frame>
      );
    case "done":
      return (
        <Frame mood="idle" title={s.done} line={phase.openFailed ? s.openFailed : s.opening} />
      );
    case "failed": {
      const { state, failure } = phase;
      return (
        <Frame mood="tired" title={s.failed} line={s.failedLine}>
          <div className="mt-6 flex flex-wrap justify-center gap-2">
            {state && (
              <Button variant="primary" onClick={() => onInstall(state)}>
                {s.retry}
              </Button>
            )}
            <Button onClick={() => host.classic().catch(() => {})}>{s.classic}</Button>
          </div>
          <Footer>
            <Details>
              {failure.code !== null && `${s.exitCode(failure.code)}\n`}
              {failure.detail}
            </Details>
          </Footer>
        </Frame>
      );
    }
  }
}

function Ready({
  state,
  onInstall,
  onOpen,
}: {
  state: SetupState;
  onInstall: (state: SetupState) => void;
  onOpen: (state: SetupState) => void;
}) {
  const s = useT().setup;
  const installed = state.installed ?? "";
  const line = {
    none: s.tagline,
    older: s.older(installed, state.version),
    same: s.same,
    newer: s.newer(installed),
  }[state.relation];
  const closesApp = state.appRunning && state.relation !== "newer";
  return (
    <Frame mood="idle" title="Botloft" line={line}>
      <div className="mt-6 flex flex-wrap justify-center gap-2">
        {state.relation === "same" || state.relation === "newer" ? (
          <Button variant="primary" onClick={() => onOpen(state)}>
            {s.open}
          </Button>
        ) : (
          <Button variant="primary" className="min-w-32" onClick={() => onInstall(state)}>
            {state.relation === "older" ? s.update : s.install}
          </Button>
        )}
        {state.relation === "same" && (
          <Button onClick={() => onInstall(state)}>{s.reinstall}</Button>
        )}
      </div>
      {closesApp && <p className="mt-3 text-ink-soft text-xs">{s.appOpen}</p>}
      <Footer>
        {state.relation !== "newer" && <p>{s.forYou}</p>}
        <Details>
          {state.folder && `${s.folder(state.folder)}\n`}
          {s.version(state.version)}
        </Details>
      </Footer>
    </Frame>
  );
}

/** The mascot, a title and a line, with what goes under them. */
function Frame({
  mood,
  title,
  line,
  children,
}: {
  mood: Mood;
  title: string;
  line: string;
  children?: ReactNode;
}) {
  return (
    <>
      <span className="inline-block animate-float">
        <BotAvatar color={BOTLOFT_COLOR} size={72} mood={mood} />
      </span>
      <h1 className="mt-3 font-semibold text-2xl tracking-tight">{title}</h1>
      <p className="mt-1 max-w-80 text-ink-soft text-sm leading-relaxed">{line}</p>
      {children}
    </>
  );
}

function Footer({ children }: { children: ReactNode }) {
  return <div className="mt-auto w-full pt-4 text-muted text-xs">{children}</div>;
}
