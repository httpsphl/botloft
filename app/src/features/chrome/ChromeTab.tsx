// The "Your Chrome" tab of the Permissions page (spec 37.2, 37.4): the
// agents the owner gave their Chrome to, giving it to another, who acts in
// it now with a way to stop them, and where the allowed sites are.

import { CirclePause, Plus } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotId, ChromeGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { AgentGroups, type Group } from "../permissions/AgentGroups";
import { ChromeRow } from "../permissions/rows";
import { GiveChromeDialog } from "./GiveChromeDialog";
import { useChromeTurn } from "./useChromeTurn";

export function ChromeTab({
  groups,
  grants,
  agents,
  only,
}: {
  groups: Group<ChromeGrant>[];
  /** Every agent's, whatever the filter: who can still be given it. */
  grants: ChromeGrant[];
  agents: Bot[];
  only: BotId | null;
}) {
  const words = useT().chrome;
  const api = useApi();
  const [giving, setGiving] = useState(false);
  const holder = useChromeTurn();
  const using = agents.find((bot) => bot.id === holder);
  const candidates = agents.filter(
    (bot) => bot.agent === "claude" && !grants.some((grant) => grant.botId === bot.id),
  );
  return (
    <div className="flex flex-col gap-5">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      <AgentGroups
        groups={groups}
        empty={words.none}
        keyOf={(grant) => grant.botId}
        row={(bot, grant) => <ChromeRow bot={bot} grant={grant} />}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          icon={Plus}
          disabled={candidates.length === 0}
          onClick={() => setGiving(true)}
        >
          {words.give}
        </Button>
      </div>
      <div className="flex flex-wrap items-center gap-2 rounded-xl bg-bubble px-3 py-2 text-sm">
        <span className="min-w-0 flex-1">{using ? words.using(using.name) : words.nobody}</span>
        {using && (
          <Button
            variant="danger"
            size="sm"
            icon={CirclePause}
            onClick={() => attempt(words.stopFailed, () => api.call("chrome.stop"))}
          >
            {words.stop}
          </Button>
        )}
      </div>
      <p className="text-muted text-xs leading-relaxed">{words.oneAtATime}</p>
      <p className="text-muted text-xs leading-relaxed">{words.sites}</p>
      {giving && (
        <GiveChromeDialog candidates={candidates} chosen={only} onClose={() => setGiving(false)} />
      )}
    </div>
  );
}
