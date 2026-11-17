// Giving the owner's own Chrome to an agent (spec 37.2): the agent, among
// those on Claude Code that do not have it yet, and the risks; the confirm
// button lights only once the owner checks that they understand them.

import { Check } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotId } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { Select } from "../../ui/Select";
import { attempt } from "../../ui/toast";
import { ChromeRisks } from "./ChromeRisks";

export function GiveChromeDialog({
  candidates,
  chosen,
  onClose,
}: {
  /** Agents on Claude Code without the Chrome. */
  candidates: Bot[];
  chosen: BotId | null;
  onClose(): void;
}) {
  const words = useT().chrome;
  const api = useApi();
  const first = candidates.find((bot) => bot.id === chosen) ?? candidates[0];
  const [botId, setBotId] = useState<BotId | null>(first?.id ?? null);
  const [understood, setUnderstood] = useState(false);
  const bot = candidates.find((each) => each.id === botId);
  const give = () => {
    if (!bot) {
      return;
    }
    onClose();
    void attempt(words.giveFailed, () =>
      api.call("chrome.set", { botId: bot.id, on: true, acceptedRisks: true }),
    );
  };
  return (
    <Dialog
      title={words.dialog.title}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {words.dialog.cancel}
          </Button>
          <Button variant="primary" icon={Check} disabled={!understood || !bot} onClick={give}>
            {words.dialog.confirm}
          </Button>
        </>
      }
    >
      {!bot && <p className="text-ink-soft text-sm">{words.dialog.noAgent}</p>}
      {bot && candidates.length > 1 && (
        <div className="w-full max-w-xs">
          <Select<BotId>
            label={words.dialog.agent}
            value={bot.id}
            options={candidates.map((each) => ({ value: each.id, label: each.name }))}
            onChange={setBotId}
          />
        </div>
      )}
      {bot && <ChromeRisks bot={bot.name} understood={understood} onUnderstood={setUnderstood} />}
    </Dialog>
  );
}
