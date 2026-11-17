// The Desktop tab of the Permissions page (spec 24.10): what each agent may
// see and use on the owner's computer, giving an app before it asks,
// stopping every agent there at once, and what is never given.

import { CirclePause, Plus } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotId, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { attempt } from "../../ui/toast";
import { AgentGroups, type Group } from "./AgentGroups";
import { GiveAppDialog } from "./GiveAppDialog";
import { DesktopGrantRow } from "./rows";

export function DesktopTab({
  groups,
  agents,
  only,
}: {
  groups: Group<DesktopGrant>[];
  agents: Bot[];
  only: BotId | null;
}) {
  const words = useT().permissions;
  const api = useApi();
  const [giving, setGiving] = useState(false);
  return (
    <div className="flex flex-col gap-5">
      <AgentGroups
        groups={groups}
        empty={words.empty.desktop}
        keyOf={(grant) => grant.id}
        row={(bot, grant) => <DesktopGrantRow bot={bot} grant={grant} />}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          icon={Plus}
          disabled={agents.length === 0}
          onClick={() => setGiving(true)}
        >
          {words.give}
        </Button>
        <Button
          variant="danger"
          size="sm"
          icon={CirclePause}
          onClick={() => attempt(words.stopAllFailed, () => api.call("desktop.stopAll"))}
        >
          {words.stopAll}
        </Button>
      </div>
      <p className="text-muted text-xs leading-relaxed">{words.stopAllHint}</p>
      <Details label={words.never}>
        <ul className="mt-2 flex list-disc flex-col gap-1 pl-5 text-ink-soft">
          {words.neverItems.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </Details>
      {giving && <GiveAppDialog agents={agents} chosen={only} onClose={() => setGiving(false)} />}
    </div>
  );
}
