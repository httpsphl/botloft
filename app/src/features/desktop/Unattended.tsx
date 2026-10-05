// The switch of a desktop grant for use while the owner is away (spec
// 24.8, 24.10): off from the start; turning it on first shows the risks,
// and the confirm button lights only once the owner checks that they
// understand them; turning it off is at once.

import { Check } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";

export function Unattended({ bot, grant, app }: { bot: Bot; grant: DesktopGrant; app: string }) {
  const words = useT().desktop.grants;
  const api = useApi();
  const [asking, setAsking] = useState(false);
  const [understood, setUnderstood] = useState(false);
  const set = (unattended: boolean) =>
    attempt(words.realFailed, () =>
      api.call("desktop.setOptions", {
        grantId: grant.id,
        unattended,
        ...(unattended ? { acceptedRisks: true } : {}),
      }),
    );
  const close = () => {
    setAsking(false);
    setUnderstood(false);
  };
  return (
    <>
      <span className="mt-1 flex items-center gap-2 text-xs">
        <Switch
          checked={grant.unattended}
          label={`${words.away}: ${app}`}
          onChange={(on) => (on ? setAsking(true) : void set(false))}
        />
        <span className="text-ink-soft">{words.away}</span>
      </span>
      {asking && (
        <Dialog
          title={words.awayTitle(bot.name, app)}
          onClose={close}
          footer={
            <>
              <Button variant="ghost" onClick={close}>
                {words.cancel}
              </Button>
              <Button
                variant="primary"
                icon={Check}
                disabled={!understood}
                onClick={() => {
                  close();
                  void set(true);
                }}
              >
                {words.awayConfirm}
              </Button>
            </>
          }
        >
          <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm">
            {words.awayRisks(bot.name, app).map((risk) => (
              <li key={risk}>{risk}</li>
            ))}
          </ul>
          <label className="mt-4 flex items-center gap-2 font-medium text-sm">
            <input
              type="checkbox"
              checked={understood}
              onChange={(event) => setUnderstood(event.target.checked)}
            />
            {words.awayUnderstood}
          </label>
        </Dialog>
      )}
    </>
  );
}
