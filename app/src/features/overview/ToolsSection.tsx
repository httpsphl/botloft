// The agent's connected tools and how they stand (spec 33.6), with the
// claude.ai tools it may use (spec 34), why one did not connect: who may
// use each is chosen in Settings > Tools, one click away.

import { useId } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { useConnections } from "../connections/useConnections";

const TONES = { connected: "ok", pending: "quiet", needs_auth: "warn", failed: "danger" } as const;

export function ToolsSection({ bot }: { bot: Bot }) {
  const t = useT();
  const w = t.overview.tools;
  const id = useId();
  const openSettingsPart = useApp((state) => state.openSettingsPart);
  const overview = useConnections();
  const mine = overview?.bots.find((link) => link.botId === bot.id);
  const servers = overview?.servers.filter((server) => mine?.serverIds.includes(server.id)) ?? [];
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h3 id={id} className="font-semibold text-sm">
        {w.title}
      </h3>
      {servers.length === 0 && bot.connectors.length === 0 ? (
        <p className="text-muted text-sm">{w.none(bot.name)}</p>
      ) : (
        <ul aria-label={w.title} className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1.5">
          {bot.connectors.map((name) => (
            <li
              key={`claude-${name}`}
              className="flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm"
            >
              <span className="min-w-0 flex-1 break-words">{name}</span>
              <span className="shrink-0 text-muted text-xs">{w.fromClaudeAi}</span>
            </li>
          ))}
          {servers.map((server) => {
            const state = mine?.states.find((each) => each.serverId === server.id);
            return (
              <li key={server.id} className="rounded-lg px-2 py-1.5 text-sm">
                <div className="flex items-center gap-2">
                  <span className="min-w-0 flex-1 break-words">{server.name}</span>
                  {state && (
                    <Badge tone={TONES[state.state]}>
                      {t.connections.states[state.state] ?? state.state}
                    </Badge>
                  )}
                </div>
                {state?.error && <Details label={t.connections.bot.reason}>{state.error}</Details>}
              </li>
            );
          })}
        </ul>
      )}
      <Button size="sm" className="self-start" onClick={() => openSettingsPart("tools")}>
        {w.change}
      </Button>
    </section>
  );
}
