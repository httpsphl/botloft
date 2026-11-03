// One tool, opened from Settings > Tools (spec 34.7): what it does, how it
// stands, and a switch for each agent that may use it, crew by crew. A tool
// of claude.ai is managed there; one the owner pasted can be edited or
// removed here.

import { Pencil, Trash2 } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, Connector, McpServer } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApi, useApp, useHost } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { CONNECTORS_PAGE } from "./BotConnectors";
import { EditToolDialog } from "./EditToolDialog";
import { ToolTile } from "./ToolTile";
import { knownTool } from "./toolCatalog";
import { useConnections } from "./useConnections";

export type Opened = { kind: "claude"; connector: Connector } | { kind: "own"; server: McpServer };

const TONES = { connected: "ok", pending: "quiet", needs_auth: "warn", failed: "danger" } as const;

export function ToolDialog({ opened, onClose }: { opened: Opened; onClose(): void }) {
  const t = useT().connections;
  const p = t.page;
  const api = useApi();
  const host = useHost();
  const putBot = useApp((state) => state.putBot);
  const crews = useApp(useShallow(crewList));
  const bots = useApp(useShallow((state) => Object.values(state.bots)));
  const overview = useConnections();
  const [asking, setAsking] = useState<"edit" | "remove" | null>(null);
  const name = opened.kind === "claude" ? opened.connector.name : opened.server.name;
  const known = knownTool(name);
  const about = opened.kind === "own" ? opened.server.description : known ? t.about[known.key] : "";
  const claude = opened.kind === "claude";
  const usable = bots.filter(
    (bot) => bot.archivedAt === null && (!claude || bot.agent === "claude"),
  );

  const uses = (bot: Bot) =>
    opened.kind === "claude"
      ? bot.connectors.includes(name)
      : (overview?.bots.find((link) => link.botId === bot.id)?.serverIds ?? []).includes(
          opened.server.id,
        );
  const set = (bot: Bot, on: boolean) => {
    if (opened.kind === "claude") {
      const names = on ? [...bot.connectors, name] : bot.connectors.filter((have) => have !== name);
      return attempt(t.claudeAi.failed, async () =>
        putBot(await api.call("bots.setConnectors", { botId: bot.id, names })),
      );
    }
    const using = overview?.bots.find((link) => link.botId === bot.id)?.serverIds ?? [];
    const serverIds = on
      ? [...using, opened.server.id]
      : using.filter((id) => id !== opened.server.id);
    return attempt(t.bot.failed, () => api.call("bot.mcp.set", { botId: bot.id, serverIds }));
  };

  return (
    <Dialog
      title={name}
      onClose={onClose}
      footer={
        opened.kind === "claude" ? (
          <Button
            onClick={() => attempt(t.claudeAi.openFailed, () => host.openUrl(CONNECTORS_PAGE))}
          >
            {p.manage}
          </Button>
        ) : (
          <>
            <Button icon={Pencil} onClick={() => setAsking("edit")}>
              {t.edit(name)}
            </Button>
            <Button variant="danger" icon={Trash2} onClick={() => setAsking("remove")}>
              {t.remove(name)}
            </Button>
          </>
        )
      }
    >
      <div className="flex flex-col gap-5">
        <div className="flex items-start gap-3">
          <ToolTile name={name} size={48} />
          <div className="flex min-w-0 flex-col gap-1">
            <p className="text-muted text-xs">{claude ? p.fromClaudeAi : p.yourOwn}</p>
            {about && <p className="text-ink-soft text-sm leading-relaxed">{about}</p>}
            {opened.kind === "claude" && (
              <div>
                <Badge tone={TONES[opened.connector.state]}>
                  {opened.connector.state === "connected"
                    ? p.connected
                    : t.states[opened.connector.state]}
                </Badge>
              </div>
            )}
          </div>
        </div>
        <section className="flex flex-col gap-2">
          <h3 className="font-semibold text-sm">{p.who}</h3>
          {claude && <p className="text-muted text-xs">{p.onlyClaude}</p>}
          {usable.length === 0 ? (
            <p className="text-muted text-sm">{p.noAgents}</p>
          ) : (
            crews.map((crew) => {
              const mine = usable.filter((bot) => bot.crewId === crew.id);
              if (mine.length === 0) {
                return null;
              }
              return (
                <div key={crew.id} className="flex flex-col gap-1">
                  <p className="px-1 text-muted text-xs">{crew.name}</p>
                  <ul
                    aria-label={crew.name}
                    className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1.5"
                  >
                    {mine.map((bot) => (
                      <li key={bot.id} className="flex items-center gap-2.5 rounded-lg px-2 py-1.5">
                        <BotAvatar color={bot.color} size={24} still />
                        <span className="min-w-0 flex-1 break-words text-sm">{bot.name}</span>
                        <Switch
                          checked={uses(bot)}
                          label={p.canUse(bot.name)}
                          onChange={(next) => set(bot, next)}
                        />
                      </li>
                    ))}
                  </ul>
                </div>
              );
            })
          )}
        </section>
      </div>
      {asking === "edit" && opened.kind === "own" && (
        <EditToolDialog server={opened.server} onClose={() => setAsking(null)} />
      )}
      {asking === "remove" && opened.kind === "own" && (
        <Confirm
          title={t.removing.title(name)}
          confirmLabel={t.removing.confirm}
          onClose={() => setAsking(null)}
          onConfirm={() =>
            attempt(t.removing.failed, async () => {
              await api.call("mcp.delete", { serverId: opened.server.id });
              onClose();
            })
          }
        >
          {t.removing.text}
        </Confirm>
      )}
    </Dialog>
  );
}
