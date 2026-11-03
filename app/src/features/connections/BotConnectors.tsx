// The owner's claude.ai tools an agent may use (spec 34.4), in its details:
// the ones it has, and the way to Settings > Tools, where each tool says who
// may use it. Signed in once on claude.ai, they need no sign-in here.

import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, ConnectorList } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { ToolTile } from "./ToolTile";

export const CONNECTORS_PAGE = "https://claude.ai/customize/connectors";

/** The account's connectors, looked for once and again on demand. */
export function useConnectorList() {
  const api = useApi();
  const [list, setList] = useState<ConnectorList | null>(null);
  const [looking, setLooking] = useState(false);
  const look = useCallback(
    async (refresh: boolean) => {
      setLooking(true);
      try {
        setList(await api.call("connectors.list", { refresh }));
      } catch {
        setList((current) => current ?? { connectors: [], checkedAt: 0 });
      } finally {
        setLooking(false);
      }
    },
    [api],
  );
  useEffect(() => {
    void look(false);
  }, [look]);
  return { list, looking, refresh: () => look(true) };
}

export function BotConnectors({ bot }: { bot: Bot }) {
  const t = useT().connections;
  const openSettingsPart = useApp((state) => state.openSettingsPart);
  if (bot.agent !== "claude") {
    return null;
  }
  return (
    <div>
      <dt className="text-muted text-xs">{t.claudeAi.title}</dt>
      <dd className="mt-1 flex flex-col gap-1.5">
        {bot.connectors.length === 0 ? (
          <p className="text-ink-soft text-xs leading-relaxed">{t.claudeAi.hint(bot.name)}</p>
        ) : (
          <ul aria-label={t.claudeAi.title} className="flex flex-col gap-1">
            {bot.connectors.map((name) => (
              <li key={name} className="flex items-center gap-2 text-sm">
                <ToolTile name={name} size={22} />
                <span className="min-w-0 flex-1 break-words">{name}</span>
              </li>
            ))}
          </ul>
        )}
        <Button size="sm" className="self-start" onClick={() => openSettingsPart("tools")}>
          {t.page.choose}
        </Button>
      </dd>
    </div>
  );
}
