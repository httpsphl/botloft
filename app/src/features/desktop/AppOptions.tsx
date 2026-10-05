// The options of the app a bot is using, right in its desktop panel (spec
// 24.9, 24.10): the real mouse and keyboard and use while the owner is
// away, the same switches as in the bot's details, where the owner looks.

import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useDesktopGrants } from "./DesktopGrants";
import { RealInput } from "./RealInput";
import { Unattended } from "./Unattended";

export function AppOptions({ bot, app }: { bot: Bot; app: string }) {
  const words = useT().desktop.panel;
  const grants = useDesktopGrants(bot.id);
  const named = app.toLowerCase();
  const grant =
    grants?.find((grant) => grant.scope === "app" && grant.appName?.toLowerCase() === named) ??
    grants?.find((grant) => grant.scope === "desktop");
  if (!grant) {
    return null;
  }
  return (
    <section
      aria-label={words.options(app)}
      className="flex flex-wrap items-center gap-x-5 gap-y-1 rounded-lg border border-line bg-sunken px-3 pt-1 pb-2"
    >
      {grant.level === "act" && <RealInput bot={bot} grant={grant} app={app} />}
      <Unattended bot={bot} grant={grant} app={app} />
    </section>
  );
}
