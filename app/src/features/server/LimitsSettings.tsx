// Settings, "This server's limits" (spec 42.4): how much of the server
// Botloft and its agents may take. The agents awake at once change here;
// the memory and the processors are the system's own caps, changed on the
// server with the command shown, or the container's.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { fileSize, processors } from "../../lib/format";
import type { MachineLimits } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Select } from "../../ui/Select";
import { Field, Section } from "../account/settingsParts";
import { useDaemonSettings } from "../account/useDaemonSettings";
import { CopyCommand } from "./CopyCommand";

/** How often the use is read again while the page is open. */
const REFRESH_MS = 10_000;
/** The agents awake at once the owner picks from; 0 is no limit. */
export const AGENT_CHOICES = [0, 1, 2, 3, 4, 6, 8, 12, 16] as const;
/** The command for the memory and the processors, as an example. */
export const LIMITS_COMMAND = "sudo botloftd limits --memory 4G --cpus 2";

export function LimitsSettings() {
  const t = useT().server.limits;
  const api = useApi();
  const { settings, change } = useDaemonSettings();
  const [limits, setLimits] = useState<MachineLimits | null>(null);
  const [failed, setFailed] = useState(false);
  const agentsMax = settings?.maxRunning ?? 0;

  useEffect(() => {
    let live = true;
    const read = async () => {
      try {
        const read = await api.call("system.limits");
        if (live) {
          setLimits(read);
          setFailed(false);
        }
      } catch {
        if (live) {
          setFailed(true);
        }
      }
    };
    read();
    const timer = setInterval(read, REFRESH_MS);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [api]);

  const options = AGENT_CHOICES.map((count) => ({
    value: String(count),
    label: count === 0 ? t.noLimit : String(count),
  }));
  // A number set by hand or with `botloftd limits` shows as it is.
  if (!AGENT_CHOICES.some((count) => count === agentsMax)) {
    options.push({ value: String(agentsMax), label: String(agentsMax) });
  }

  return (
    <Section title={t.title}>
      <p className="text-ink-soft text-sm leading-relaxed">{t.hint}</p>
      {failed && !limits && <p className="text-danger text-sm">{t.failed}</p>}
      {limits && (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
          <dt className="text-muted">{t.memory}</dt>
          <dd>{memoryText(limits, t)}</dd>
          <dt className="text-muted">{t.processors}</dt>
          <dd>
            {limits.cpuLimitPercent === undefined
              ? t.processorsOpen(limits.cpus)
              : t.processorsOf(processors(limits.cpuLimitPercent), limits.cpus)}
          </dd>
        </dl>
      )}
      <Field label={t.agents} hint={t.agentsHint(limits?.agentsAwake ?? 0)}>
        <div className="max-w-xs">
          <Select
            label={t.agents}
            value={String(agentsMax)}
            options={options}
            onChange={(value) => change({ maxRunning: Number(value) })}
          />
        </div>
      </Field>
      {limits &&
        (limits.changeable ? (
          <div className="flex flex-col gap-2">
            <p className="text-muted text-xs leading-relaxed">{t.change}</p>
            <CopyCommand command={LIMITS_COMMAND} />
          </div>
        ) : (
          <p className="text-muted text-xs leading-relaxed">{t.container}</p>
        ))}
    </Section>
  );
}

type Words = ReturnType<typeof useT>["server"]["limits"];

function memoryText(limits: MachineLimits, t: Words): string {
  const { memoryUsed: used, memoryLimit: limit, memoryTotal: total } = limits;
  if (total === undefined) {
    return limit === undefined ? t.unknown : fileSize(limit);
  }
  if (limit !== undefined && used !== undefined) {
    return t.memoryOf(fileSize(used), fileSize(limit), fileSize(total));
  }
  if (used !== undefined) {
    return t.memoryUsed(fileSize(used), fileSize(total));
  }
  return t.memoryOpen(fileSize(total));
}
