// The switch of a desktop grant for the owner's real mouse and keyboard
// (spec 24.7, 24.10): off from the start; turning it on first says, in a
// dialog, what changes and how to stop it; turning it off is at once.

import { Check } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";

export function RealInput({ bot, grant, app }: { bot: Bot; grant: DesktopGrant; app: string }) {
  const words = useT().desktop.grants;
  const api = useApi();
  const [asking, setAsking] = useState(false);
  const set = (realInput: boolean) =>
    attempt(words.realFailed, () =>
      api.call("desktop.setOptions", { grantId: grant.id, realInput }),
    );
  return (
    <>
      <span className="mt-1 flex items-center gap-2 text-xs">
        <Switch
          checked={grant.realInput}
          label={`${words.real}: ${app}`}
          onChange={(on) => (on ? setAsking(true) : void set(false))}
        />
        <span className="text-ink-soft">{words.real}</span>
      </span>
      {asking && (
        <Dialog
          title={words.realTitle(bot.name, app)}
          onClose={() => setAsking(false)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setAsking(false)}>
                {words.cancel}
              </Button>
              <Button
                variant="primary"
                icon={Check}
                onClick={() => {
                  setAsking(false);
                  void set(true);
                }}
              >
                {words.realConfirm}
              </Button>
            </>
          }
        >
          <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm">
            {words.realPoints(bot.name).map((point) => (
              <li key={point}>{point}</li>
            ))}
          </ul>
        </Dialog>
      )}
    </>
  );
}
