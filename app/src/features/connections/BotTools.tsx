// What a bot can use of the owner's connected tools (spec 25.3, 25.5), in
// its details: a switch for each tool, and whether the tool came up when
// the bot started.

import { useT } from "../../i18n";
import type { Bot, McpServer, McpServerState } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Details } from "../../ui/Details";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";
import { useConnections } from "./useConnections";

const TONES = { connected: "ok", pending: "quiet", needs_auth: "warn", failed: "danger" } as const;

function State({ state }: { state: McpServerState | undefined }) {
  const t = useT().connections;
  if (!state) {
    return null;
  }
  return (
    <div className="shrink-0 text-right">
      <Badge tone={TONES[state.state]}>{t.states[state.state] ?? state.state}</Badge>
    </div>
  );
}

export function BotTools({ bot }: { bot: Bot }) {
  const t = useT().connections;
  const api = useApi();
  const overview = useConnections();
  if (!overview) {
    return null;
  }
  const mine = overview.bots.find((link) => link.botId === bot.id);
  const using = mine?.serverIds ?? [];
  const set = (server: McpServer, on: boolean) => {
    const next = on ? [...using, server.id] : using.filter((id) => id !== server.id);
    return attempt(t.bot.failed, () => api.call("bot.mcp.set", { botId: bot.id, serverIds: next }));
  };

  return (
    <div>
      <dt className="text-muted text-xs">{t.bot.title}</dt>
      <dd className="mt-1">
        {overview.servers.length === 0 ? (
          <p className="text-ink-soft text-xs leading-relaxed">{t.bot.none}</p>
        ) : (
          <ul className="flex flex-col gap-1.5">
            {overview.servers.map((server) => {
              const on = using.includes(server.id);
              const state = on
                ? mine?.states.find((each) => each.serverId === server.id)
                : undefined;
              return (
                <li
                  key={server.id}
                  className="rounded-lg border border-line bg-sunken px-2.5 py-1.5"
                >
                  <div className="flex items-center gap-2">
                    <span className="min-w-0 flex-1 truncate text-sm">{server.name}</span>
                    <State state={state} />
                    <Switch
                      checked={on}
                      label={t.bot.toggle(server.name, bot.name)}
                      onChange={(next) => set(server, next)}
                    />
                  </div>
                  {state?.error && <Details label={t.bot.reason}>{state.error}</Details>}
                </li>
              );
            })}
          </ul>
        )}
      </dd>
    </div>
  );
}
