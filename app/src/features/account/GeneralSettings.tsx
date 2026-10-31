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
import { AgentPrograms } from "./AgentPrograms";
import { Field, Section, Toggle } from "./settingsParts";
import { useDaemonSettings } from "./useDaemonSettings";

const NO_AGENTS: AgentKind[] = [];
/** The times the owner chooses from, in minutes; 0 never (spec 7.7). */
const SLEEP_CHOICES = [15, 30, 60, 0] as const;

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
      <Section title={s.experimentalAgents}>
        <Toggle
          label={s.showExperimental}
          hint={settings?.showExperimentalAgents ? s.showExperimentalOn : s.showExperimentalOff}
          checked={settings?.showExperimentalAgents ?? false}
          disabled={!settings}
          onChange={(on) => change({ showExperimentalAgents: on })}
        />
        {settings?.showExperimentalAgents && <AgentPrograms />}
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
        <Field label={s.sleepAfter} hint={s.sleepAfterHint}>
          <div className="max-w-xs">
            <Select
              label={s.sleepAfter}
              value={String(settings?.sleepAfterMinutes ?? 30)}
              options={SLEEP_CHOICES.map((minutes) => ({
                value: String(minutes),
                label: s.sleepAfterChoices[minutes],
              }))}
              onChange={(minutes) => change({ sleepAfterMinutes: Number(minutes) })}
            />
          </div>
        </Field>
      </Section>
    </>
  );
}
