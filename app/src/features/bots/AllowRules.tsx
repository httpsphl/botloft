// What the bot may do without asking (spec 10.1), in its details: each
// "Allow always" the owner gave, with a button that makes the bot ask again.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { AllowRule, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { RuleRow } from "../permissions/rows";

/** The bot's rules, read when shown and kept current by `bot.rules`. */
function useAllowRules(botId: Bot["id"]): AllowRule[] | null {
  const api = useApi();
  const [rules, setRules] = useState<AllowRule[] | null>(null);
  useEffect(() => {
    let alive = true;
    setRules(null);
    const stop = api.subscribe((event) => {
      if (event.name === "bot.rules" && event.params.botId === botId) {
        setRules(event.params.rules);
      }
    });
    api.call("rules.list", { botId }).then(
      (listed) => alive && setRules((current) => current ?? listed),
      // An older daemon has no rules: the list stays empty.
      () => alive && setRules([]),
    );
    return () => {
      alive = false;
      stop();
    };
  }, [api, botId]);
  return rules;
}

export function AllowRules({ bot }: { bot: Bot }) {
  const words = useT().bots.details;
  const rules = useAllowRules(bot.id);
  return (
    <div>
      <dt className="text-muted text-xs">{words.always}</dt>
      <dd className="mt-1">
        {rules !== null && rules.length === 0 && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.alwaysNone(bot.name)}</p>
        )}
        {rules !== null && rules.length > 0 && (
          <ul className="flex flex-col gap-1.5">
            {rules.map((rule) => (
              <RuleRow key={rule.id} rule={rule} />
            ))}
          </ul>
        )}
      </dd>
    </div>
  );
}
