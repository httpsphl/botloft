// Settings, "Island" (spec 31.9): whether the island shows at the top of
// the screen, whether it steps aside with Botloft in front, calling it by
// holding the right Ctrl, on which screen, and the cheer. Windows only.

import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Select } from "../../ui/Select";
import { Section, Toggle } from "./settingsParts";

type Hold = "500" | "1000" | "2000";

export function IslandSettings() {
  const s = useT().account.settings;
  const island = usePref(prefs.island);
  const hideWithApp = usePref(prefs.islandHideWithApp);
  const summon = usePref(prefs.islandSummon);
  const hold = usePref(prefs.islandHold);
  const screen = usePref(prefs.islandScreen);
  const cheer = usePref(prefs.islandCheer);
  return (
    <Section title={s.islandPart}>
      <Toggle label={s.island} hint={s.islandHint} checked={island} onChange={prefs.island.set} />
      {island && (
        <>
          <Toggle
            label={s.islandSummon}
            hint={s.islandSummonHint}
            checked={summon}
            onChange={prefs.islandSummon.set}
          />
          {summon && (
            <Select<Hold>
              label={s.islandHold}
              value={hold}
              options={(["500", "1000", "2000"] as const).map((value) => ({
                value,
                label: s.islandHolds[value],
              }))}
              onChange={prefs.islandHold.set}
            />
          )}
          <Toggle
            label={s.islandHideWithApp}
            hint={s.islandHideWithAppHint}
            checked={hideWithApp}
            onChange={prefs.islandHideWithApp.set}
          />
          <Select<"primary" | "mouse">
            label={s.islandScreen}
            value={screen}
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
    </Section>
  );
}
