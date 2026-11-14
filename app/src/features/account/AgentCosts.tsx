// What the agents cost over a period (spec 8.7), in the usage dialog: the
// API price of what they used, per agent, model, day or routine, each with
// its part of the total. The owner's plan does not charge it: it is there
// to compare. Each agent shows its crew, as two crews can have agents with
// the same name.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { dollars, planPercent } from "../../lib/format";
import type { BotTokens, CostGroup } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Callout } from "../../ui/Callout";
import { Choices } from "../../ui/Choices";
import { BotAvatar } from "../bots/BotAvatar";
import { agentRows, type CostRow, type CostView, groupRows, totalCost } from "./costRows";

type Period = "hour" | "today" | "week" | "month" | "all";

const HOUR_MS = 3_600_000;
const DAY_MS = 24 * HOUR_MS;

/** When `period` starts, in Unix ms. */
export function periodStart(period: Period, now = Date.now()): number {
  switch (period) {
    case "hour":
      return now - HOUR_MS;
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

const GROUPING = { models: "model", days: "day", routines: "routine" } as const;

/** What the daemon summed, for the view it was asked for. */
type Found =
  | { view: "agents"; bots: BotTokens[] }
  | { view: Exclude<CostView, "agents">; groups: CostGroup[] };

export function AgentCosts() {
  const u = useT().account.usage.costs;
  const api = useApi();
  const [period, setPeriod] = useState<Period>("today");
  const [view, setView] = useState<CostView>("agents");
  const [found, setFound] = useState<Found | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    setFailed(null);
    const since = periodStart(period);
    const read: Promise<Found> =
      view === "agents"
        ? api.call("usage.tokens", { since }).then((bots) => ({ view, bots }))
        : api
            .call("usage.costs", { since, by: GROUPING[view] })
            .then((groups) => ({ view, groups }));
    read.then(
      (result) => alive && setFound(result),
      (error) => alive && setFailed(errorText(error)),
    );
    return () => {
      alive = false;
    };
  }, [api, period, view]);

  const periods = (["hour", "today", "week", "month", "all"] as const).map((value) => ({
    value,
    label: u.periods[value],
  }));
  const views = (["agents", "models", "days", "routines"] as const).map((value) => ({
    value,
    label: u.views[value],
  }));
  const rows: CostRow[] | null =
    found === null
      ? null
      : found.view === "agents"
        ? agentRows(found.bots, u)
        : groupRows(found.view, found.groups, u);
  const total = totalCost(rows ?? []);
  const faces = found?.view === "agents" || found?.view === "routines";

  return (
    <section className="flex flex-col gap-3 border-line border-t pt-4">
      <h3 className="font-semibold">{u.title}</h3>
      <p className="text-ink-soft leading-relaxed">{u.intro}</p>
      <Choices<Period> label={u.period} value={period} options={periods} onChange={setPeriod} />
      <Choices<CostView> label={u.view} value={view} options={views} onChange={setView} />
      {failed ? (
        <Callout tone="danger" title={u.failed}>
          {failed}
        </Callout>
      ) : rows === null ? null : rows.length === 0 ? (
        <p className="text-muted">{u.empty}</p>
      ) : (
        <>
          <ul aria-label={u.title} className="flex flex-col gap-3">
            {rows.map((row) => (
              <Row key={row.key} row={row} total={total} faces={faces} />
            ))}
            {rows.length > 1 && (
              <li className="flex items-baseline justify-between gap-3 border-line border-t pt-3 font-semibold">
                <span>{u.total}</span>
                <span className="tabular-nums">{dollars(total)}</span>
              </li>
            )}
          </ul>
          <p className="text-muted text-xs leading-relaxed">{u.note}</p>
        </>
      )}
    </section>
  );
}

function Row({ row, total, faces }: { row: CostRow; total: number; faces: boolean }) {
  const u = useT().account.usage.costs;
  const share = row.cost !== null && total > 0 ? row.cost / total : null;
  return (
    <li className="flex items-center gap-3">
      {faces &&
        (row.color === null ? (
          <span aria-hidden className="size-6 shrink-0" />
        ) : (
          <BotAvatar color={row.color} size={24} />
        ))}
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <div className="flex items-baseline justify-between gap-3">
          <span className="truncate font-medium">
            {row.label}
            {row.archived && <span className="ml-1.5 text-muted text-xs">{u.archived}</span>}
          </span>
          <span className="shrink-0 tabular-nums">
            {row.cost === null ? u.noPrice(row.tokens) : dollars(row.cost)}
          </span>
        </div>
        {share !== null && (
          <div aria-hidden className="h-1 w-full bg-sunken">
            <div
              className="h-full origin-left animate-grow bg-work"
              style={{ width: `${share * 100}%` }}
            />
          </div>
        )}
        {(row.detail !== null || share !== null) && (
          <span className="text-muted text-xs">
            {[row.detail, share === null ? null : u.share(planPercent(share))]
              .filter(Boolean)
              .join(" · ")}
          </span>
        )}
      </div>
    </li>
  );
}
