import { CircleCheck, CircleX, LoaderCircle, Plus } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { CrewDialog } from "../crews/CrewDialog";
import { ClaudeCodeHelp } from "./ClaudeCodeHelp";
import { SignInButton } from "./SignIn";

/** First run: what is ready, what is not, and the first crew. */
export function Welcome() {
  const system = useApp((state) => state.system);
  const [creating, setCreating] = useState(false);

  let claude: ReactNode;
  if (!system || (system.claudeVersion === null && system.runtimeError === null)) {
    claude = <Check state="pending" title="Claude Code" detail="Checking…" />;
  } else if (system.runtimeError) {
    claude = (
      <Check
        state="bad"
        title="Claude Code"
        detail={<ClaudeCodeHelp error={system.runtimeError} />}
      />
    );
  } else {
    claude = <Check state="ok" title="Claude Code" detail={`Version ${system.claudeVersion}`} />;
  }

  // Only once Claude Code is there: signing in needs it.
  let account: ReactNode = null;
  if (system && !system.runtimeError && system.claudeVersion !== null) {
    if (system.claudeSignedIn === true) {
      account = <Check state="ok" title="Claude account" detail="Signed in." />;
    } else if (system.claudeSignedIn === false) {
      account = (
        <Check
          state="bad"
          title="Claude account"
          detail={
            <>
              <p>Your bots work with your Claude account. Sign in once and they are ready.</p>
              <SignInButton />
            </>
          }
        />
      );
    } else {
      account = <Check state="pending" title="Claude account" detail="Checking…" />;
    }
  }

  return (
    <main className="min-h-0 flex-1 overflow-y-auto p-8">
      <div className="mx-auto max-w-xl">
        <h1 className="font-semibold text-2xl tracking-tight">Welcome to Botloft</h1>
        <p className="mt-1 text-ink-soft leading-relaxed">
          A crew is a group of Claude Code bots that keep running, message each other and share a
          folder. Start with one crew and a bot or two.
        </p>
        <ul className="mt-6 flex flex-col border border-line bg-panel">
          <Check
            state="ok"
            title="Botloft"
            detail="Running in the background. It starts with Windows, so your bots keep working after you close this window."
          />
          {claude}
          {account}
        </ul>
        <div className="mt-4 border border-line bg-panel p-4 text-sm leading-relaxed">
          <p className="font-semibold">Bots ask before they change things</p>
          <p className="mt-1 text-ink-soft">
            When a bot wants to run a command or edit a file, it asks in its chat and waits for you
            to allow or deny it.
          </p>
        </div>
        <Button className="mt-6" variant="primary" icon={Plus} onClick={() => setCreating(true)}>
          Create your first crew
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
            : {state === "ok" ? "ready" : state === "bad" ? "not ready" : "checking"}
          </span>
        </p>
        <div className="text-ink-soft" data-selectable>
          {detail}
        </div>
      </div>
    </li>
  );
}
