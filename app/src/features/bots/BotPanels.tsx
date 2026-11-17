// The notices over a bot's chat (spec 15.1), about a paused crew, a missing
// program or a bot that needs the owner.

import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Callout } from "../../ui/Callout";
import { SignInButton } from "../onboarding/SignIn";
import type { stateView } from "./BotStateBadge";

export function Notices({
  bot,
  crew,
  view,
}: {
  bot: Bot;
  crew: Crew;
  view: ReturnType<typeof stateView>;
}) {
  const t = useT();
  const notices = [];
  const checks = useApp((state) => state.system?.agentChecks);
  // An agent that is not on the computer is restarted over and over: say why.
  const missing =
    bot.agent !== "claude" &&
    checks?.some((check) => check.agent === bot.agent && check.path === null) === true;
  if (missing) {
    const name = t.bots.dialog.agent.names[bot.agent];
    notices.push(
      <Callout key="program" tone="warn" title={t.bots.notices.programMissing(name)}>
        {t.bots.notices.programMissingBody}
      </Callout>,
    );
  }
  if (crew.paused && !bot.paused) {
    notices.push(
      <Callout key="crew" title={t.bots.notices.crewPaused(crew.name)}>
        {t.bots.notices.crewPausedBody}
      </Callout>,
    );
  }
  // An approval shows in the chat itself; the rest needs a word up here.
  if ((view.tone === "warn" || view.tone === "danger") && bot.state !== "needs_approval") {
    notices.push(
      <Callout key="state" tone={view.tone} title={view.label}>
        {view.hint}
        {bot.state === "auth_error" && <SignInButton />}
      </Callout>,
    );
  }
  if (notices.length === 0) {
    return null;
  }
  return <div className="flex flex-col gap-2 px-5 py-3">{notices}</div>;
}
