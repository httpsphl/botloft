// Giving an agent an app that is open now, or the whole desktop, before it
// asks (spec 24.10): the agent, the app among the open ones, and what it may
// do. The whole desktop shows its risks first and waits for the owner's
// check, as in the agent's details.

import { Check, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotId, DesktopApp, DesktopLevel } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Choices } from "../../ui/Choices";
import { Dialog } from "../../ui/Dialog";
import { Select } from "../../ui/Select";
import { attempt } from "../../ui/toast";
import { Field } from "../account/settingsParts";
import { WholeRisks } from "../desktop/WholeDesktop";

/** The choice that stands for the whole desktop instead of one app. */
const WHOLE = "";

export function GiveAppDialog({
  agents,
  chosen,
  onClose,
}: {
  /** The agents that may be given something, in the order to list them. */
  agents: Bot[];
  chosen: BotId | null;
  onClose(): void;
}) {
  const t = useT();
  const words = t.permissions;
  const api = useApi();
  const crews = useApp((state) => state.crews);
  const [botId, setBotId] = useState<BotId | null>(chosen ?? agents[0]?.id ?? null);
  const [apps, setApps] = useState<DesktopApp[] | null>(null);
  const [path, setPath] = useState<string | null>(null);
  const [level, setLevel] = useState<DesktopLevel>("see");
  const [understood, setUnderstood] = useState(false);
  const read = useCallback(() => {
    setApps(null);
    api.call("desktop.apps").then(
      (open) => {
        setApps(open);
        setPath((current) =>
          current !== null && (current === WHOLE || open.some((app) => app.path === current))
            ? current
            : (open[0]?.path ?? WHOLE),
        );
      },
      () => setApps([]),
    );
  }, [api]);
  useEffect(read, [read]);
  const bot = agents.find((agent) => agent.id === botId) ?? null;
  const whole = path === WHOLE;
  const give = (to: Bot, what: string) =>
    attempt(words.giveFailed, () =>
      what === WHOLE
        ? api.call("desktop.grantWhole", { botId: to.id, level, acceptedRisks: true })
        : api.call("desktop.grantApp", { botId: to.id, path: what, level }),
    );

  return (
    <Dialog
      title={words.giveTitle}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {words.cancel}
          </Button>
          <Button
            variant="primary"
            icon={Check}
            disabled={bot === null || path === null || (whole && !understood)}
            onClick={() => {
              if (bot && path !== null) {
                onClose();
                void give(bot, path);
              }
            }}
          >
            {words.giveConfirm}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <Field label={words.agent}>
          <Select<BotId>
            label={words.agent}
            value={botId ?? ""}
            options={agents.map((agent) => ({
              value: agent.id,
              label: [agent.name, crews[agent.crewId]?.name].filter(Boolean).join(" · "),
            }))}
            onChange={setBotId}
          />
        </Field>
        <Field label={words.giveApp} hint={words.giveHint}>
          <div role="radiogroup" aria-label={words.giveApp} className="flex flex-col gap-1">
            {apps?.length === 0 && <p className="text-ink-soft text-sm">{words.giveNoApps}</p>}
            {[...(apps ?? []), null].map((app) => {
              const value = app?.path ?? WHOLE;
              return (
                <label
                  key={value}
                  className={`flex cursor-pointer items-start gap-2 rounded-lg border px-2.5 py-1.5 text-sm ${
                    path === value ? "border-accent bg-sunken" : "border-line hover:bg-sunken"
                  }`}
                >
                  <input
                    type="radio"
                    name="give-app"
                    className="mt-1"
                    checked={path === value}
                    onChange={() => setPath(value)}
                  />
                  <span className="min-w-0">
                    <span className="block">{app?.name ?? t.desktop.grants.whole}</span>
                    {app && (
                      <span className="block break-all font-mono text-muted text-xs">
                        {app.path}
                      </span>
                    )}
                  </span>
                </label>
              );
            })}
          </div>
          <Button variant="ghost" size="sm" icon={RefreshCw} onClick={read}>
            {words.giveRefresh}
          </Button>
        </Field>
        <Field label={words.giveLevel}>
          <Choices<DesktopLevel>
            label={words.giveLevel}
            value={level}
            options={[
              { value: "see", label: t.desktop.grants.see },
              { value: "act", label: t.desktop.grants.act },
            ]}
            onChange={setLevel}
          />
        </Field>
      </div>
      {whole && bot && (
        <WholeRisks bot={bot.name} understood={understood} onUnderstood={setUnderstood} />
      )}
    </Dialog>
  );
}
