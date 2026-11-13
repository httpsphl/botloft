// "Every skill for every agent" at the top of Settings > Skills (spec 38.4):
// turning it on asks first, saying how many skills there are and how many
// run things without asking; turning it off is at once.

import { useId, useState } from "react";
import { useT } from "../../i18n";
import type { Skill } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { warningsOf } from "./SkillWarnings";

export function AllForEveryone({
  on,
  skills,
  onChange,
}: {
  on: boolean;
  skills: Skill[];
  onChange(on: boolean): Promise<unknown>;
}) {
  const t = useT();
  const words = t.skills;
  const id = useId();
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);
  const risky = skills.filter((skill) => warningsOf(skill, words.warnings).length > 0).length;
  const give = async () => {
    setBusy(true);
    try {
      await onChange(true);
    } finally {
      setBusy(false);
      setAsking(false);
    }
  };
  return (
    <div className="flex items-center gap-3 rounded-xl bg-bubble px-3.5 py-3">
      <div className="min-w-0 flex-1">
        <p id={id} className="font-medium text-sm">
          {words.everyone.all}
        </p>
        <p className="text-muted text-xs leading-relaxed">{words.everyone.allHint}</p>
      </div>
      <Switch
        checked={on}
        labelledBy={id}
        onChange={(next) => (next ? setAsking(true) : void onChange(false))}
      />
      {asking && (
        <Dialog
          title={words.everyone.allAsk(skills.length)}
          onClose={() => setAsking(false)}
          footer={
            <>
              <Button onClick={() => setAsking(false)}>{t.common.cancel}</Button>
              <Button variant="primary" disabled={busy} onClick={() => void give()}>
                {words.everyone.allGive}
              </Button>
            </>
          }
        >
          <div className="flex flex-col gap-2 text-ink-soft text-sm leading-relaxed">
            <p>{words.everyone.allBody}</p>
            {risky > 0 && <p className="text-warn">{words.everyone.allRisky(risky)}</p>}
          </div>
        </Dialog>
      )}
    </div>
  );
}
