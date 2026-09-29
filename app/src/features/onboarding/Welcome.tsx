import { CircleCheck, CircleX, LoaderCircle, Plus } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import { useWhenClosed } from "../../shell/closing";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { useDaemonSettings } from "../account/useDaemonSettings";
import { BotAvatar } from "../bots/BotAvatar";
import { CrewDialog } from "../crews/CrewDialog";
import { ClaudeCodeHelp } from "./ClaudeCodeHelp";
import { SignInButton } from "./SignIn";

/** First run: what is ready, what is not, and the first crew. */
export function Welcome() {
  const w = useT().onboarding.welcome;
  const system = useApp((state) => state.system);
  const [creating, setCreating] = useState(false);
  const whenClosed = useWhenClosed();
  const { settings } = useDaemonSettings();
  const running =
    whenClosed === "stop"
      ? w.runningWhileOpen
      : settings?.startWithWindows === false
        ? w.runningNotAtStart
        : w.running;

  let claude: ReactNode;
  if (!system || (system.claudeVersion === null && system.runtimeError === null)) {
    claude = <Check state="pending" title={w.claudeCode} detail={w.checking} />;
  } else if (system.runtimeError) {
    claude = (
      <Check
        state="bad"
        title={w.claudeCode}
        detail={<ClaudeCodeHelp error={system.runtimeError} />}
      />
    );
  } else {
    claude = (
      <Check state="ok" title={w.claudeCode} detail={w.version(system.claudeVersion ?? "")} />
    );
  }

  // Only once Claude Code is there: signing in needs it.
  let account: ReactNode = null;
  if (system && !system.runtimeError && system.claudeVersion !== null) {
    if (system.claudeSignedIn === true) {
      account = <Check state="ok" title={w.account} detail={w.signedIn} />;
    } else if (system.claudeSignedIn === false) {
      account = (
        <Check
          state="bad"
          title={w.account}
          detail={
            <>
              <p>{w.signedOut}</p>
              <SignInButton />
            </>
          }
        />
      );
    } else {
      account = <Check state="pending" title={w.account} detail={w.checking} />;
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-8">
      <div className="mx-auto max-w-xl">
        <span className="mb-3 inline-block animate-float">
          <BotAvatar color="#ff7a59" size={56} mood="idle" />
        </span>
        <h1 className="font-semibold text-2xl tracking-tight">{w.title}</h1>
        <p className="mt-1 text-ink-soft leading-relaxed">{w.intro}</p>
        <ul className="mt-6 flex flex-col overflow-hidden rounded-xl border border-line bg-panel">
          <Check state="ok" title={w.botloft} detail={running} />
          {claude}
          {account}
        </ul>
        <div className="mt-4 rounded-xl border border-line bg-panel p-4 text-sm leading-relaxed">
          <p className="font-semibold">{w.askFirstTitle}</p>
          <p className="mt-1 text-ink-soft">{w.askFirstBody}</p>
        </div>
        <Button className="mt-6" variant="primary" icon={Plus} onClick={() => setCreating(true)}>
          {w.createCrew}
        </Button>
      </div>
      {creating && <CrewDialog onClose={() => setCreating(false)} />}
    </main>
  );
}

function Check({
  state,
  title,
  detail,
}: {
  state: "ok" | "bad" | "pending";
  title: string;
  detail: ReactNode;
}) {
  const w = useT().onboarding.welcome;
  const Icon = state === "ok" ? CircleCheck : state === "bad" ? CircleX : LoaderCircle;
  const tone = state === "ok" ? "text-ok" : state === "bad" ? "text-danger" : "text-muted";
  return (
    <li className="flex items-start gap-3 border-line border-b px-4 py-3 last:border-b-0">
      <Icon
        aria-hidden
        size={18}
        className={`mt-0.5 shrink-0 ${tone} ${state === "pending" ? "animate-spin" : ""}`}
      />
      <div className="text-sm">
        <p className="font-semibold">
          {title}
          <span className="sr-only">
            : {state === "ok" ? w.ready : state === "bad" ? w.notReady : w.stillChecking}
          </span>
        </p>
        <div className="text-ink-soft" data-selectable>
          {detail}
        </div>
      </div>
    </li>
  );
}
