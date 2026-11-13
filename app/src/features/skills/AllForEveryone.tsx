// "Every skill for every agent" at the top of Settings > Skills (spec 38.4),
// and the same for plugins (spec 41.5): turning it on asks first, saying
// how many there are and how many run things without asking; turning it
// off is at once.

import { useId, useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";

/** The words of one page's switch and question. */
export interface EveryoneWords {
  all: string;
  allHint: string;
  allAsk(count: number): string;
  allBody: string;
  allRisky(count: number): string;
  allGive: string;
}

export function AllForEveryone({
  on,
  count,
  risky,
  words,
  onChange,
}: {
  on: boolean;
  /** How many there are to give. */
  count: number;
  /** How many of them run things without asking. */
  risky: number;
  words: EveryoneWords;
  onChange(on: boolean): Promise<unknown>;
}) {
  const t = useT();
  const id = useId();
  const [asking, setAsking] = useState(false);
  const [busy, setBusy] = useState(false);
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
          {words.all}
        </p>
        <p className="text-muted text-xs leading-relaxed">{words.allHint}</p>
      </div>
      <Switch
        checked={on}
        labelledBy={id}
        onChange={(next) => (next ? setAsking(true) : void onChange(false))}
      />
      {asking && (
        <Dialog
          title={words.allAsk(count)}
          onClose={() => setAsking(false)}
          footer={
            <>
              <Button onClick={() => setAsking(false)}>{t.common.cancel}</Button>
              <Button variant="primary" disabled={busy} onClick={() => void give()}>
                {words.allGive}
              </Button>
            </>
          }
        >
          <div className="flex flex-col gap-2 text-ink-soft text-sm leading-relaxed">
            <p>{words.allBody}</p>
            {risky > 0 && <p className="text-warn">{words.allRisky(risky)}</p>}
          </div>
        </Dialog>
      )}
    </div>
  );
}
