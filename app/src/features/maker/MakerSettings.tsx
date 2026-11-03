// Settings, the Maker (spec 36.2, 36.4): show it on the left, and let it
// build without asking, which asks for a confirmation first.

import { useState } from "react";
import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Confirm } from "../../ui/Confirm";
import { Section, Toggle } from "../account/settingsParts";
import { useDaemonSettings } from "../account/useDaemonSettings";

export function MakerSettings() {
  const s = useT().maker.settings;
  const shown = usePref(prefs.showMaker);
  const { settings, change } = useDaemonSettings();
  const [asking, setAsking] = useState(false);
  const withoutAsking = settings?.makerBuildsWithoutAsking ?? false;
  return (
    <Section title={s.title}>
      <Toggle
        label={s.show}
        hint={shown ? s.showOn : s.showOff}
        checked={shown}
        onChange={prefs.showMaker.set}
      />
      <Toggle
        label={s.withoutAsking}
        hint={withoutAsking ? s.withoutAskingOn : s.withoutAskingOff}
        checked={withoutAsking}
        disabled={!settings}
        onChange={(on) => (on ? setAsking(true) : change({ makerBuildsWithoutAsking: false }))}
      />
      {asking && (
        <Confirm
          title={s.confirmTitle}
          confirmLabel={s.confirm}
          onClose={() => setAsking(false)}
          onConfirm={async () => change({ makerBuildsWithoutAsking: true })}
        >
          {s.confirmBody}
        </Confirm>
      )}
    </Section>
  );
}
