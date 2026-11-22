// The strip at the top about Claude's sign-in for the agents (spec 7.3,
// 42.9): signed out, or ending within days. Botloft on this computer signs
// in from here; Botloft elsewhere (a server), in that computer's terminal.

import { SignInButton } from "../features/onboarding/SignIn";
import { CopyCommand } from "../features/server/CopyCommand";
import { useT } from "../i18n";
import { day } from "../lib/format";
import type { SignInEnds } from "../lib/protocol.gen";
import { useApp } from "../store/context";
import { Callout } from "../ui/Callout";
import { useServer } from "./server";

/** How long before the earlier end date the strip shows. */
const WARN_BEFORE_MS = 5 * 24 * 60 * 60 * 1000;

/** The command that signs the agents in again where Botloft runs. */
export function signInCommand(onServer: boolean): string {
  return onServer ? "sudo -u botloft-agents -i claude auth login" : "claude auth login";
}

/** Whether the sign-in ends within the warning's days of `now`. */
export function endsSoon(ends: SignInEnds | undefined, now: number): boolean {
  return ends !== undefined && now >= ends.from - WARN_BEFORE_MS;
}

/** `signedOut`: Claude Code is signed out, and the agents need it. */
export function ClaudeSignInNotice({ signedOut }: { signedOut: boolean }) {
  const t = useT().shell.signIn;
  const ends = useApp((state) => state.system?.claudeSignInEnds);
  const onServer = useApp((state) => state.system?.server ?? false);
  const elsewhere = useServer() !== null || onServer;
  const how = elsewhere ? (
    <div className="mt-2 flex flex-col gap-2">
      <p>{t.inTerminal}</p>
      <CopyCommand command={signInCommand(onServer)} />
    </div>
  ) : (
    <SignInButton />
  );

  if (signedOut) {
    return (
      <div className="border-line border-b p-3">
        <Callout tone="danger" title={t.title}>
          {elsewhere ? t.endedElsewhere : t.body}
          {how}
        </Callout>
      </div>
    );
  }
  if (!ends || !endsSoon(ends, Date.now())) {
    return null;
  }
  const now = ends.from === ends.by;
  return (
    <div className="border-line border-b p-3">
      <Callout tone="warn" title={t.endsTitle}>
        {now ? t.endsNow : t.endsBody(day(ends.from), day(ends.by))}
        {how}
      </Callout>
    </div>
  );
}
