// Giving a bot the owner's whole desktop (spec 24.2, 24.10): a button in
// the bot's details that opens the risks screen, with the level to give;
// the confirm button lights only once the owner checks that they
// understand the risks. Never asked for in the chat.

import { Check, Monitor } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopLevel } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Choices } from "../../ui/Choices";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";

export function WholeDesktop({ bot }: { bot: Bot }) {
  const words = useT().desktop.grants;
  const api = useApi();
  const [asking, setAsking] = useState(false);
  const [level, setLevel] = useState<DesktopLevel>("see");
  const [understood, setUnderstood] = useState(false);
  const close = () => {
    setAsking(false);
    setUnderstood(false);
    setLevel("see");
  };
  const give = () =>
    attempt(words.wholeFailed, () =>
      api.call("desktop.grantWhole", { botId: bot.id, level, acceptedRisks: true }),
    );
  return (
    <>
      <Button variant="secondary" size="sm" icon={Monitor} onClick={() => setAsking(true)}>
        {words.giveWhole}
      </Button>
      {asking && (
        <Dialog
          title={words.wholeTitle(bot.name)}
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
                  void give();
                }}
              >
                {words.wholeConfirm}
              </Button>
            </>
          }
        >
          <Choices<DesktopLevel>
            label={words.wholeLevel}
            value={level}
            options={[
              { value: "see", label: words.see },
              { value: "act", label: words.act },
            ]}
            onChange={setLevel}
          />
          <ul className="mt-4 flex list-disc flex-col gap-1.5 pl-5 text-sm">
            {words.wholeRisks(bot.name).map((risk) => (
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
