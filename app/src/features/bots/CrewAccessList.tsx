// The other crews a bot may reach for good (spec 10.4), in its details:
// each crew, or one bot of it, with a button that takes it back. Read when
// shown and again when a request to reach a crew is answered.

import { Network, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, CrewAccess } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { CREW_ACCESS_TOOL } from "../chat/CrewAccessCard";

function useCrewAccess(botId: Bot["id"]) {
  const api = useApi();
  const [list, setList] = useState<CrewAccess[] | null>(null);
  useEffect(() => {
    let alive = true;
    const read = () =>
      api.call("crewAccess.list", { botId }).then(
        (listed) => alive && setList(listed),
        // An older daemon has no access to other crews: the list stays empty.
        () => alive && setList([]),
      );
    setList(null);
    read();
    const stop = api.subscribe((event) => {
      if (event.name !== "chat.item" || event.params.item.botId !== botId) {
        return;
      }
      const body = event.params.item.body;
      if (body.kind === "approval" && body.toolName === CREW_ACCESS_TOOL) {
        read();
      }
    });
    return () => {
      alive = false;
      stop();
    };
  }, [api, botId]);
  return [list, setList] as const;
}

export function CrewAccessList({ bot }: { bot: Bot }) {
  const words = useT().bots.details;
  const api = useApi();
  const [list, setList] = useCrewAccess(bot.id);
  return (
    <div>
      <dt className="text-muted text-xs">{words.crews}</dt>
      <dd className="mt-1">
        {list !== null && list.length === 0 && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.crewsNone(bot.name)}</p>
        )}
        {list !== null && list.length > 0 && (
          <ul className="flex flex-col gap-1.5">
            {list.map((access) => {
              const what = access.targetName
                ? words.crewsBot(access.targetName, access.crewName)
                : words.crewsWhole(access.crewName);
              return (
                <li
                  key={access.id}
                  className="flex items-center gap-2 rounded-lg border border-line bg-sunken py-1.5 pr-1 pl-2.5"
                >
                  <Network aria-hidden size={14} className="shrink-0 text-muted" />
                  <span className="min-w-0 flex-1 text-xs">
                    <span className="block">{what}</span>
                    <span className="block text-ink-soft">
                      {[
                        access.talk && words.crewsKinds.talk,
                        access.edit ? words.crewsKinds.edit : access.read && words.crewsKinds.read,
                      ]
                        .filter(Boolean)
                        .join(" · ")}
                    </span>
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={X}
                    label={words.crewsRemove(what)}
                    onClick={() =>
                      attempt(words.crewsRemoveFailed, async () => {
                        setList(await api.call("crewAccess.revoke", { accessId: access.id }));
                      })
                    }
                  />
                </li>
              );
            })}
          </ul>
        )}
      </dd>
    </div>
  );
}
