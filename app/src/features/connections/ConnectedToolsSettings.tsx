// Settings, "Connected tools" (spec 25.7): the tools the owner connected,
// which bots use each, and the way to connect, edit or remove one.

import { Pencil, Plug, Trash2 } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { McpOverview, McpServer } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { attempt } from "../../ui/toast";
import { Section } from "../account/settingsParts";
import { AddToolDialog } from "./AddToolDialog";
import { EditToolDialog } from "./EditToolDialog";
import { useConnections } from "./useConnections";

type Asking = { edit: McpServer } | { remove: McpServer } | "add" | null;

export function ConnectedToolsSettings() {
  const t = useT().connections;
  const api = useApi();
  const overview = useConnections();
  const bots = useApp((state) => state.bots);
  const [asking, setAsking] = useState<Asking>(null);

  const users = (server: McpServer, of: McpOverview): string[] =>
    of.bots
      .filter((link) => link.serverIds.includes(server.id))
      .map((link) => bots[link.botId]?.name)
      .filter((name): name is string => name !== undefined);

  return (
    <Section title={t.title}>
      <p className="text-ink-soft text-sm leading-relaxed">{t.intro}</p>
      {overview && overview.servers.length === 0 && <p className="text-muted text-sm">{t.none}</p>}
      {overview && overview.servers.length > 0 && (
        <ul className="flex flex-col gap-2">
          {overview.servers.map((server) => {
            const names = users(server, overview);
            return (
              <li
                key={server.id}
                className="flex items-start gap-3 rounded-lg border border-line bg-sunken py-2 pr-1.5 pl-3"
              >
                <Plug aria-hidden size={15} className="mt-0.5 shrink-0 text-ink-soft" />
                <span className="min-w-0 flex-1">
                  <span className="block font-medium text-sm">{server.name}</span>
                  <span className="block text-muted text-xs">
                    {t.kinds[server.kind] ?? server.kind}
                  </span>
                  {server.description && (
                    <span className="mt-0.5 block text-ink-soft text-xs">{server.description}</span>
                  )}
                  <span className="mt-0.5 block text-ink-soft text-xs">
                    {names.length > 0 ? t.usedBy(names.join(", ")) : t.unused}
                  </span>
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  icon={Pencil}
                  label={t.edit(server.name)}
                  onClick={() => setAsking({ edit: server })}
                />
                <Button
                  variant="ghost"
                  size="sm"
                  icon={Trash2}
                  label={t.remove(server.name)}
                  onClick={() => setAsking({ remove: server })}
                />
              </li>
            );
          })}
        </ul>
      )}
      <Button icon={Plug} className="self-start" onClick={() => setAsking("add")}>
        {t.add}
      </Button>
      {asking === "add" && <AddToolDialog onClose={() => setAsking(null)} />}
      {asking && typeof asking === "object" && "edit" in asking && (
        <EditToolDialog server={asking.edit} onClose={() => setAsking(null)} />
      )}
      {asking && typeof asking === "object" && "remove" in asking && (
        <Confirm
          title={t.removing.title(asking.remove.name)}
          confirmLabel={t.removing.confirm}
          onClose={() => setAsking(null)}
          onConfirm={() =>
            attempt(t.removing.failed, () => api.call("mcp.delete", { serverId: asking.remove.id }))
          }
        >
          {t.removing.text}
        </Confirm>
      )}
    </Section>
  );
}
