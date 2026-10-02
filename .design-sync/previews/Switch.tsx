import { setLocaleChoice, Switch } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

function Row({ label, hint, initial }: { label: string; hint: string; initial: boolean }) {
  const [on, setOn] = useState(initial);
  return (
    <div className="flex items-start justify-between gap-6 py-2">
      <div>
        <p className="font-medium text-ink text-sm">{label}</p>
        <p className="text-muted text-xs">{hint}</p>
      </div>
      <Switch checked={on} onChange={setOn} label={label} />
    </div>
  );
}

export function SettingsRows() {
  return (
    <div className="flex max-w-md flex-col divide-y divide-line p-4">
      <Row
        label="Keep working after closing Botloft"
        hint="Your bots carry on after the window closes."
        initial
      />
      <Row
        label="Keep the computer awake while bots work"
        hint="It still sleeps when you close the lid or choose Sleep."
        initial={false}
      />
    </div>
  );
}

export function States() {
  return (
    <div className="flex items-center gap-4 p-4">
      <Switch checked onChange={() => {}} label="On" />
      <Switch checked={false} onChange={() => {}} label="Off" />
      <Switch checked disabled onChange={() => {}} label="On, disabled" />
      <Switch checked={false} disabled onChange={() => {}} label="Off, disabled" />
    </div>
  );
}
