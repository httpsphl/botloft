// Tokens each bot used over a period (spec 8.7), in the usage dialog: the
// new text read plus the replies written, with the conversation sent
// again after a pause shown apart. Each bot shows its crew, as two crews can have
// bots with the same name.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { reloadedTokens, tokens, usedTokens } from "../../lib/format";
import type { BotTokens } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Callout } from "../../ui/Callout";
import { Choices } from "../../ui/Choices";
import { BotAvatar } from "../bots/BotAvatar";

type Period = "today" | "week" | "month" | "all";

const DAY_MS = 86_400_000;

/** When `period` starts, in Unix ms. */
export function periodStart(period: Period, now = Date.now()): number {
  switch (period) {
    case "today":
      return new Date(now).setHours(0, 0, 0, 0);
    case "week":
      return now - 7 * DAY_MS;
    case "month":
      return now - 30 * DAY_MS;
    case "all":
      return 0;
  }
}

export function TokensByBot() {
  const u = useT().account.usage.tokens;
  const api = useApi();
  const [period, setPeriod] = useState<Period>("today");
  const [bots, setBots] = useState<BotTokens[] | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    setFailed(null);
    api.call("usage.tokens", { since: periodStart(period) }).then(
      (list) => alive && setBots(list),
      (error) => alive && setFailed(errorText(error)),
    );
    return () => {
      alive = false;
    };
  }, [api, period]);

  const periods = (["today", "week", "month", "all"] as const).map((value) => ({
    value,
    label: u.periods[value],
  }));
  const most = Math.max(1, ...(bots ?? []).map((bot) => usedTokens(bot.tokens)));

  return (
    <section className="flex flex-col gap-3 border-line border-t pt-4">
      <h3 className="font-semibold">{u.title}</h3>
      <p className="text-ink-soft leading-relaxed">{u.intro}</p>
      <Choices<Period> label={u.period} value={period} options={periods} onChange={setPeriod} />
      {failed ? (
        <Callout tone="danger" title={u.failed}>
          {failed}
        </Callout>
      ) : bots === null ? null : bots.length === 0 ? (
        <p className="text-muted">{u.empty}</p>
      ) : (
        <ul aria-label={u.title} className="flex flex-col gap-3">
          {bots.map((bot) => (
            <Row key={bot.botId} bot={bot} most={most} />
          ))}
          {bots.length > 1 && <Total bots={bots} />}
        </ul>
      )}
    </section>
  );
}

function Row({ bot, most }: { bot: BotTokens; most: number }) {
  const u = useT().account.usage.tokens;
  const used = usedTokens(bot.tokens);
  return (
    <li className="flex items-center gap-3">
      <BotAvatar color={bot.color} size={24} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <div className="flex items-baseline justify-between gap-3">
          <span className="truncate font-medium">
            {bot.name}
            {bot.archived && <span className="ml-1.5 text-muted text-xs">{u.archived}</span>}
          </span>
          <span className="shrink-0 tabular-nums">{tokens(used)}</span>
        </div>
        <div aria-hidden className="h-1 w-full bg-sunken">
          <div
            className="h-full origin-left animate-grow bg-work"
            style={{ width: `${(used / most) * 100}%` }}
          />
        </div>
        <span className="text-muted text-xs">
          {u.detail(bot.crew, bot.turns, reloadedTokens(bot.tokens.reloaded))}
        </span>
      </div>
    </li>
  );
}

function Total({ bots }: { bots: BotTokens[] }) {
  const u = useT().account.usage.tokens;
  const used = bots.reduce((sum, bot) => sum + usedTokens(bot.tokens), 0);
  const turns = bots.reduce((sum, bot) => sum + bot.turns, 0);
  const reloaded = bots.reduce((sum, bot) => sum + bot.tokens.reloaded, 0);
  return (
    <li className="flex flex-col gap-1 border-line border-t pt-3">
      <div className="flex items-baseline justify-between gap-3 font-semibold">
        <span>{u.total}</span>
        <span className="tabular-nums">{tokens(used)}</span>
      </div>
      <span className="text-muted text-xs">{u.detail(null, turns, reloadedTokens(reloaded))}</span>
    </li>
  );
}
