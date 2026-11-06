// Settings, "General": how long the chats stay (spec 12.1). For good by
// default; a shorter time deletes what is older at once, so it asks first.

import { useState } from "react";
import { useT } from "../../i18n";
import { Confirm } from "../../ui/Confirm";
import { Select } from "../../ui/Select";
import { Field, Section } from "./settingsParts";
import { useDaemonSettings } from "./useDaemonSettings";

/** The days the owner chooses from; 0 keeps everything (spec 12.1). */
export const HISTORY_CHOICES = [0, 30, 90, 365] as const;
type HistoryChoice = (typeof HISTORY_CHOICES)[number];

/** Whether keeping `next` days deletes something `current` kept. */
function deletesMore(current: number, next: number): boolean {
  return next > 0 && (current === 0 || next < current);
}

export function HistorySettings() {
  const s = useT().account.settings;
  const { settings, change } = useDaemonSettings();
  const [asking, setAsking] = useState<HistoryChoice | null>(null);
  const current = settings?.historyDays ?? 0;
  const known = HISTORY_CHOICES.find((days) => days === current);
  const options = HISTORY_CHOICES.map((days) => ({
    value: String(days),
    label: s.historyChoices[days],
  }));
  // A number set in `config.toml` by hand shows as it is.
  if (known === undefined) {
    options.push({ value: String(current), label: s.historyDaysCustom(current) });
  }

  const choose = (value: string) => {
    const days = Number(value) as HistoryChoice;
    if (deletesMore(current, days)) {
      setAsking(days);
    } else {
      change({ historyDays: days });
    }
  };

  return (
    <Section title={s.history}>
      <Field label={s.historyKeep} hint={s.historyHint}>
        <div className="max-w-xs">
          <Select
            label={s.historyKeep}
            value={String(current)}
            options={options}
            onChange={choose}
          />
        </div>
      </Field>
      {asking !== null && (
        <Confirm
          title={s.historyConfirmTitle}
          confirmLabel={s.historyConfirm}
          onClose={() => setAsking(null)}
          onConfirm={async () => change({ historyDays: asking })}
        >
          {s.historyConfirmBody(s.historyChoices[asking])}
        </Confirm>
      )}
    </Section>
  );
}
