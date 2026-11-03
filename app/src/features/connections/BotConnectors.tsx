// The owner's claude.ai tools an agent may use (spec 34.4), in its details:
// a switch for each tool of the account, with how it stands. Signed in
// once on claude.ai, they need no sign-in here.

import { RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, ConnectorList } from "../../lib/protocol.gen";
import { useApi, useApp, useHost } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";

const TONES = { connected: "ok", pending: "quiet", needs_auth: "warn", failed: "danger" } as const;
const CONNECTORS_PAGE = "https://claude.ai/customize/connectors";

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
  const t = useT().connections.claudeAi;
  const states = useT().connections.states;
  const api = useApi();
  const host = useHost();
  const putBot = useApp((state) => state.putBot);
  const { list, looking, refresh } = useConnectorList();
  if (bot.agent !== "claude") {
    return null;
  }
  const set = (name: string, on: boolean) => {
    const names = on ? [...bot.connectors, name] : bot.connectors.filter((have) => have !== name);
    return attempt(t.failed, async () =>
      putBot(await api.call("bots.setConnectors", { botId: bot.id, names })),
    );
  };
  return (
    <div>
      <dt className="flex items-center justify-between gap-2 text-muted text-xs">
        {t.title}
        <Button
          variant="ghost"
          size="sm"
          icon={RefreshCw}
          label={t.refresh}
          disabled={looking}
          aria-busy={looking}
          onClick={() => void refresh()}
        />
      </dt>
      <dd className="mt-1 flex flex-col gap-1.5">
        {list === null ? (
          <p className="text-ink-soft text-xs">{t.looking}</p>
        ) : list.connectors.length === 0 ? (
          <p className="text-ink-soft text-xs leading-relaxed">{t.none}</p>
        ) : (
          <ul className="flex flex-col gap-1.5">
            {list.connectors.map((connector) => {
              const on = bot.connectors.includes(connector.name);
              return (
                <li
                  key={connector.name}
                  className="flex items-center gap-2 rounded-xl bg-bubble py-2 pr-2 pl-3"
                >
                  <span className="min-w-0 flex-1 break-words text-sm">{connector.name}</span>
                  {connector.state !== "connected" && (
                    <Badge tone={TONES[connector.state]}>{states[connector.state]}</Badge>
                  )}
                  <Switch
                    checked={on}
                    label={t.toggle(connector.name, bot.name)}
                    onChange={(next) => set(connector.name, next)}
                  />
                </li>
              );
            })}
          </ul>
        )}
        <p className="text-muted text-xs leading-relaxed">{t.hint(bot.name)}</p>
        <Button
          size="sm"
          className="self-start"
          onClick={() => attempt(t.openFailed, () => host.openUrl(CONNECTORS_PAGE))}
        >
          {t.manage}
        </Button>
      </dd>
    </div>
  );
}
