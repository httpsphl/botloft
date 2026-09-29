// Settings, "Notifications" (spec 15.1): when Botloft tells the owner, with
// a Windows notification while it is not in front, and whether a sound
// goes with it.

import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Section, Toggle } from "./settingsParts";

export function AlertsSettings() {
  const s = useT().account.settings;
  const needs = usePref(prefs.notifyNeeds);
  const done = usePref(prefs.notifyDone);
  const sound = usePref(prefs.sound);
  const tray = usePref(prefs.tray);
  const keep = usePref(prefs.whenClosed) === "keep";
  const nearClock = tray && keep;
  return (
    <Section title={s.alerts}>
      <Toggle
        label={s.notifyNeeds}
        hint={s.notifyNeedsHint}
        checked={needs}
        onChange={prefs.notifyNeeds.set}
      />
      <Toggle
        label={s.notifyDone}
        hint={s.notifyDoneHint}
        checked={done}
        onChange={prefs.notifyDone.set}
      />
      <Toggle label={s.sound} hint={s.soundHint} checked={sound} onChange={prefs.sound.set} />
      {!nearClock && <p className="text-muted text-xs leading-relaxed">{s.alertsNeedTray}</p>}
    </Section>
  );
}
