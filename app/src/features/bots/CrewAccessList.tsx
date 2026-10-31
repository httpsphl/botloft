// The other crews a bot may reach for good (spec 10.4), in its details:
// each crew, or one bot of it, with a button that takes it back. Read when
// shown and again when a request to reach a crew is answered.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, CrewAccess } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { CREW_ACCESS_TOOL } from "../chat/CrewAccessCard";
import { CrewAccessRow } from "../permissions/rows";

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
            {list.map((access) => (
              <CrewAccessRow key={access.id} access={access} onRevoked={setList} />
            ))}
          </ul>
        )}
      </dd>
    </div>
  );
}
