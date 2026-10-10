// Settings, "General": the language, what Botloft does in the background
// and when Windows starts (spec 15.1).

import {
  LOCALES,
  type LocaleChoice,
  setLocaleChoice,
  systemLocale,
  useLocale,
  useT,
} from "../../i18n";
import type { AgentKind } from "../../lib/protocol.gen";
import { SYSTEM } from "../../lib/system";
import { prefs, usePref } from "../../shell/prefs";
import { useApp } from "../../store/context";
import { Select } from "../../ui/Select";
import { Section, Toggle } from "./settingsParts";
import { useDaemonSettings } from "./useDaemonSettings";

const NO_AGENTS: AgentKind[] = [];

export function GeneralSettings() {
  const t = useT();
  const s = t.account.settings;
  const whenClosed = usePref(prefs.whenClosed);
  const tray = usePref(prefs.tray);
  const openAtSignIn = usePref(prefs.openAtSignIn);
  const nearClock = whenClosed === "keep" && tray;
  const { settings, change } = useDaemonSettings();
  const enabledAgents = useApp((state) => state.system?.enabledAgents ?? NO_AGENTS);
  const startWithWindows = settings?.startWithWindows ?? true;
  const { choice: locale } = useLocale();
  const systemName = LOCALES.find((entry) => entry.id === systemLocale())?.name ?? "English";
  const languages: { value: LocaleChoice; label: string }[] = [
    { value: "system", label: t.shell.language.system(systemName) },
    ...LOCALES.map((entry) => ({ value: entry.id as LocaleChoice, label: entry.name })),
  ];

  return (
    <>
      <Section title={s.language}>
        <Select label={s.language} value={locale} options={languages} onChange={setLocaleChoice} />
      </Section>
      {settings && enabledAgents.length > 1 && (
        <Section title={s.defaultAgent}>
          <Select
            label={s.defaultAgent}
            value={settings.defaultAgent}
            options={enabledAgents.map((agent) => ({
              value: agent,
              label: t.bots.dialog.agent.names[agent],
            }))}
            onChange={(agent) => change({ defaultAgent: agent })}
          />
          <p className="text-muted text-xs leading-relaxed">{s.defaultAgentHint}</p>
        </Section>
      )}
      <Section title={s.background}>
        <Toggle
          label={s.keepWorking}
          hint={whenClosed === "keep" ? s.keepWorkingOn : s.keepWorkingOff}
          checked={whenClosed === "keep"}
          onChange={(on) => prefs.whenClosed.set(on ? "keep" : "stop")}
        />
        {whenClosed === "keep" && (
          <Toggle
            label={s.tray}
            hint={tray ? s.trayOn : s.trayOff}
            checked={tray}
            onChange={prefs.tray.set}
          />
        )}
        <Toggle
          label={s.startWithSystem(SYSTEM)}
          hint={startWithWindows ? s.startWithSystemOn(SYSTEM) : s.startWithSystemOff}
          checked={startWithWindows}
          disabled={!settings}
          onChange={(on) => change({ startWithWindows: on })}
        />
        {startWithWindows && settings && (
          <Toggle
            label={s.openAtSignIn(SYSTEM)}
            hint={
              openAtSignIn
                ? s.openAtSignInOn(SYSTEM)
                : nearClock
                  ? s.openAtSignInNearClock
                  : s.openAtSignInOff
            }
            checked={openAtSignIn}
            onChange={prefs.openAtSignIn.set}
          />
        )}
        <Toggle
          label={s.keepAwake}
          hint={s.keepAwakeHint}
          checked={settings?.keepAwake ?? true}
          disabled={!settings}
          onChange={(on) => change({ keepAwake: on })}
        />
      </Section>
    </>
  );
}
