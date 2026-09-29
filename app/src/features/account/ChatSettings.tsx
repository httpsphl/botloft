// Settings, "Chat": how a message is sent, whether the panel beside the
// chat follows what the bot starts doing (spec 15.1), and how long a
// request for permission waits for the owner (spec 10.1).

import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Choices } from "../../ui/Choices";
import { Field, Section, Toggle } from "./settingsParts";
import { useDaemonSettings } from "./useDaemonSettings";

/** The waits offered, in minutes; one set by hand in config.toml joins them. */
const WAITS = [15, 30, 60, 120, 240, 480];

export function ChatSettings() {
  const s = useT().account.settings;
  const enterSends = usePref(prefs.enterSends);
  const followBot = usePref(prefs.followBot);
  const { settings, change } = useDaemonSettings();
  const current = settings?.approvalWaitMinutes;
  const waits =
    current && !WAITS.includes(current) ? [...WAITS, current].sort((a, b) => a - b) : WAITS;
  return (
    <Section title={s.chat}>
      <Toggle
        label={s.enterSends}
        hint={enterSends ? s.enterSendsOn : s.enterSendsOff}
        checked={enterSends}
        onChange={prefs.enterSends.set}
      />
      <Toggle
        label={s.followBot}
        hint={followBot ? s.followBotOn : s.followBotOff}
        checked={followBot}
        onChange={prefs.followBot.set}
      />
      {settings && (
        <Field label={s.approvalWait} hint={s.approvalWaitHint}>
          <Choices<number>
            label={s.approvalWait}
            value={settings.approvalWaitMinutes}
            options={waits.map((minutes) => ({ value: minutes, label: s.waitFor(minutes) }))}
            onChange={(minutes) => change({ approvalWaitMinutes: minutes })}
          />
        </Field>
      )}
    </Section>
  );
}
