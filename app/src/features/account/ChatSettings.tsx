// Settings, "Chat": how a message is sent, and whether the panel beside
// the chat follows what the bot starts doing (spec 15.1).

import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Section, Toggle } from "./settingsParts";

export function ChatSettings() {
  const s = useT().account.settings;
  const enterSends = usePref(prefs.enterSends);
  const followBot = usePref(prefs.followBot);
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
    </Section>
  );
}
