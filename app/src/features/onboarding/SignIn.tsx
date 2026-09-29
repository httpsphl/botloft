// Signing in to Claude from the app (spec 15.2): a window opens with
// Claude Code's own sign-in, the browser does the rest, and the daemon
// starts the bots that were waiting as soon as it sees the sign-in.

import { LogIn } from "lucide-react";
import { useState } from "react";
import { errorText } from "../../lib/api";
import type { SystemStatus } from "../../lib/protocol.gen";
import { useApi, useApp, useAppStore, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";

/** How long to wait for the daemon to see a sign-in that just finished. */
const CONFIRM_TRIES = 20;
const CONFIRM_EVERY_MS = 1000;

type SignInState =
  | { stage: "idle" }
  | { stage: "waiting" }
  | { stage: "failed"; error: string | null };

export function useClaudeSignIn() {
  const host = useHost();
  const api = useApi();
  const store = useAppStore();
  const claudePath = useApp((state) => state.system?.claudePath ?? null);
  const [state, setState] = useState<SignInState>({ stage: "idle" });

  const signIn = async () => {
    if (!claudePath) {
      return;
    }
    setState({ stage: "waiting" });
    try {
      if (!(await host.signInToClaude(claudePath))) {
        setState({ stage: "failed", error: null });
        return;
      }
      // Ask for a check now instead of at the next one, then watch for it.
      let system: SystemStatus = await api.call("system.refresh");
      for (let tries = 0; system.claudeSignedIn !== true && tries < CONFIRM_TRIES; tries += 1) {
        await new Promise((resolve) => setTimeout(resolve, CONFIRM_EVERY_MS));
        system = await api.call("system.status");
      }
      store.setState({ system });
      setState({ stage: "idle" });
    } catch (error) {
      setState({ stage: "failed", error: errorText(error) });
    }
  };

  return { signIn, state, available: claudePath !== null };
}

/** "Sign in to Claude", with what happens while the window is open. */
export function SignInButton() {
  const { signIn, state, available } = useClaudeSignIn();
  if (!available) {
    return null;
  }
  const waiting = state.stage === "waiting";
  return (
    <div className="mt-2 flex flex-col items-start gap-2">
      <Button variant="primary" size="sm" icon={LogIn} onClick={signIn} disabled={waiting}>
        {waiting ? "Waiting for you to sign in…" : "Sign in to Claude"}
      </Button>
      {waiting && (
        <p role="status" className="text-ink-soft text-xs">
          A window opened with Claude's sign-in. Finish it in your browser; Botloft continues by
          itself.
        </p>
      )}
      {state.stage === "failed" && (
        <Callout tone="danger" title="The sign-in didn't finish">
          Try again, and finish the sign-in in the browser before closing its window.
          {state.error && <Details>{state.error}</Details>}
        </Callout>
      )}
    </div>
  );
}
