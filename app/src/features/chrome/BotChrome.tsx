// The owner's Chrome in one agent's details (spec 37.2): whether it has
// it, with its X, or the way to give it.

import { AppWindow } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { ChromeRow } from "../permissions/rows";
import { usePermissions } from "../permissions/usePermissions";
import { GiveChromeDialog } from "./GiveChromeDialog";

export function BotChrome({ bot }: { bot: Bot }) {
  const words = useT().chrome.details;
  const all = usePermissions();
  const [giving, setGiving] = useState(false);
  if (all === null) {
    return null;
  }
  const grant = all.chrome.find((each) => each.botId === bot.id);
  return (
    <div>
      <dt className="text-muted text-xs">{words.label}</dt>
      <dd className="mt-1 flex flex-col gap-1.5">
        {grant && (
          <ul className="flex flex-col gap-1.5">
            <ChromeRow bot={bot} grant={grant} />
          </ul>
        )}
        {!grant && bot.agent !== "claude" && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.notClaude}</p>
        )}
        {!grant && bot.agent === "claude" && (
          <>
            <p className="text-ink-soft text-xs leading-relaxed">{words.off(bot.name)}</p>
            <div>
              <Button
                variant="secondary"
                size="sm"
                icon={AppWindow}
                onClick={() => setGiving(true)}
              >
                {words.give}
              </Button>
            </div>
          </>
        )}
      </dd>
      {giving && (
        <GiveChromeDialog candidates={[bot]} chosen={bot.id} onClose={() => setGiving(false)} />
      )}
    </div>
  );
}
