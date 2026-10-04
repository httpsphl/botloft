// What sits around a bot's chat besides the chat (spec 15.1): the notices
// over it, about a paused crew or a bot that needs the owner, and the panel
// with the bot's details.

import { X } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { SidePanel } from "../../ui/SidePanel";
import { DesktopGrants } from "../desktop/DesktopGrants";
import { SignInButton } from "../onboarding/SignIn";
import { AllowRules } from "./AllowRules";
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
  return <div className="flex flex-col gap-2 border-line border-b px-5 py-3">{notices}</div>;
}

export function Details({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const words = useT().bots.details;
  return (
    <SidePanel label={words.title(bot.name)} name="details" defaultWidth={320}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">{words.title(bot.name)}</h2>
        <Button variant="ghost" size="sm" icon={X} label={words.close} onClick={onClose} />
      </header>
      <dl className="flex min-h-0 flex-col gap-4 overflow-y-auto p-4 text-sm">
        <div>
          <dt className="text-muted text-xs">{words.role}</dt>
          <dd className="mt-0.5 text-ink-soft">{bot.role || words.noRole}</dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.folder}</dt>
          <dd className="mt-0.5 break-all font-mono text-xs" data-selectable>
            {bot.workspace}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.process}</dt>
          <dd className="mt-0.5 font-mono text-xs">
            {bot.generation === null ? words.notStarted : words.generation(bot.generation)}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.instructions}</dt>
          <dd className="mt-0.5 whitespace-pre-wrap text-ink-soft" data-selectable>
            {bot.instructions || words.noInstructions}
          </dd>
        </div>
        <AllowRules bot={bot} />
        <DesktopGrants bot={bot} />
      </dl>
    </SidePanel>
  );
}
