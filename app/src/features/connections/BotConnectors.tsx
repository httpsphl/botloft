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

/** How old a list may be before the Tools page looks again on its own. */
const STALE_MS = 60_000;

/** Whether a list may be out of date: something in it is not connected, or
 * it has none, and the last look was a while ago (spec 34.3). */
function mayHaveChanged(list: ConnectorList): boolean {
  const unsettled =
    list.connectors.length === 0 || list.connectors.some((tool) => tool.state !== "connected");
  return unsettled && Date.now() - list.checkedAt > STALE_MS;
}

/** The account's connectors, looked for once and again on demand. With
 * `fresh`, it looks again by itself when the kept list may be out of date:
 * the owner may just have signed in to a tool on claude.ai. */
export function useConnectorList({ fresh = false }: { fresh?: boolean } = {}) {
  const api = useApi();
  const [list, setList] = useState<ConnectorList | null>(null);
  const [looking, setLooking] = useState(false);
  const look = useCallback(
    async (refresh: boolean) => {
      setLooking(true);
      try {
        const found = await api.call("connectors.list", { refresh });
        setList(found);
        return found;
      } catch {
        setList((current) => current ?? { connectors: [], checkedAt: 0 });
        return null;
      } finally {
        setLooking(false);
      }
    },
    [api],
  );
  useEffect(() => {
    void look(false).then((kept) => {
      if (fresh && kept && mayHaveChanged(kept)) {
        void look(true);
      }
    });
  }, [look, fresh]);
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
