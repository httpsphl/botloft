// Settings, "Notifications" (spec 15.1): when Botloft tells the owner, with
// a Windows notification while it is not in front, and whether a sound
// goes with it; on Windows, the island at the top of the screen (spec 31.9).

import { useT } from "../../i18n";
import { SYSTEM } from "../../lib/system";
import { prefs, usePref } from "../../shell/prefs";
import { Select } from "../../ui/Select";
import { Section, Toggle } from "./settingsParts";

export function AlertsSettings() {
  const s = useT().account.settings;
  const needs = usePref(prefs.notifyNeeds);
  const done = usePref(prefs.notifyDone);
  const failed = usePref(prefs.notifyRoutineFailed);
  const sound = usePref(prefs.sound);
  const appSounds = usePref(prefs.appSounds);
  const replies = usePref(prefs.markReplies);
  const tray = usePref(prefs.tray);
  const keep = usePref(prefs.whenClosed) === "keep";
  const nearClock = tray && keep;
  const island = usePref(prefs.island);
  const islandScreen = usePref(prefs.islandScreen);
  const cheer = usePref(prefs.islandCheer);
  return (
    <Section title={s.alerts}>
      {SYSTEM === "Windows" && (
        <>
          <Toggle
            label={s.island}
            hint={s.islandHint}
            checked={island}
            onChange={prefs.island.set}
          />
          {island && (
            <>
              <Select<"primary" | "mouse">
                label={s.islandScreen}
                value={islandScreen}
                options={[
                  { value: "primary", label: s.islandScreens.primary },
                  { value: "mouse", label: s.islandScreens.mouse },
                ]}
                onChange={prefs.islandScreen.set}
              />
              <Toggle
                label={s.islandCheer}
                hint={s.islandCheerHint}
                checked={cheer}
                onChange={prefs.islandCheer.set}
              />
            </>
          )}
        </>
      )}
      <Toggle
        label={s.notifyNeeds}
        hint={s.notifyNeedsHint(SYSTEM)}
        checked={needs}
        onChange={prefs.notifyNeeds.set}
      />
      <Toggle
        label={s.notifyDone}
        hint={s.notifyDoneHint}
        checked={done}
        onChange={prefs.notifyDone.set}
      />
      <Toggle
        label={s.notifyRoutineFailed}
        hint={s.notifyRoutineFailedHint}
        checked={failed}
        onChange={prefs.notifyRoutineFailed.set}
      />
      {/* The mark on the icon is a taskbar overlay, which only Windows has. */}
      {SYSTEM === "Windows" && (
        <Toggle
          label={s.markReplies}
          hint={s.markRepliesHint}
          checked={replies}
          onChange={prefs.markReplies.set}
        />
      )}
      <Toggle label={s.sound} hint={s.soundHint} checked={sound} onChange={prefs.sound.set} />
      <Toggle
        label={s.appSounds}
        hint={s.appSoundsHint}
        checked={appSounds}
        onChange={prefs.appSounds.set}
      />
      {!nearClock && <p className="text-muted text-xs leading-relaxed">{s.alertsNeedTray}</p>}
    </Section>
  );
}
