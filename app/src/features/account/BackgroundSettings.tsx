// Settings, "In the background" (spec 15.1): whether the bots keep working
// after the window closes, whether Botloft starts with Windows, and
// whether the computer stays awake while they work.

import { useT } from "../../i18n";
import { setWhenClosed, useWhenClosed } from "../../shell/closing";
import { Section, Toggle } from "./settingsParts";
import { useDaemonSettings } from "./useDaemonSettings";

export function BackgroundSettings() {
  const s = useT().account.settings;
  const whenClosed = useWhenClosed();
  const { settings, change } = useDaemonSettings();
  const startWithWindows = settings?.startWithWindows ?? true;

  return (
    <Section title={s.background}>
      <Toggle
        label={s.keepWorking}
        hint={whenClosed === "keep" ? s.keepWorkingOn : s.keepWorkingOff}
        checked={whenClosed === "keep"}
        onChange={(on) => setWhenClosed(on ? "keep" : "stop")}
      />
      <Toggle
        label={s.startWithWindows}
        hint={startWithWindows ? s.startWithWindowsOn : s.startWithWindowsOff}
        checked={startWithWindows}
        disabled={!settings}
        onChange={(on) => change({ startWithWindows: on })}
      />
      <Toggle
        label={s.keepAwake}
        hint={s.keepAwakeHint}
        checked={settings?.keepAwake ?? true}
        disabled={!settings}
        onChange={(on) => change({ keepAwake: on })}
      />
    </Section>
  );
}
