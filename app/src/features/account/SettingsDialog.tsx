// Settings (spec 15.1): theme, size and language, which used to sit in the
// title bar, and the versions for when something needs reporting.

import type { ReactNode } from "react";
import {
  LOCALES,
  type LocaleChoice,
  setLocaleChoice,
  systemLocale,
  useLocale,
  useT,
} from "../../i18n";
import { setTheme, type ThemeChoice, useTheme } from "../../shell/theme";
import { DEFAULT_ZOOM, setZoom, useZoom, ZOOM_LEVELS, type ZoomLevel } from "../../shell/zoom";
import { useApp } from "../../store/context";
import { Choices } from "../../ui/Choices";
import { Dialog } from "../../ui/Dialog";

export function SettingsDialog({ onClose }: { onClose(): void }) {
  const t = useT();
  const s = t.account.settings;
  const { choice: theme } = useTheme();
  const zoom = useZoom();
  const { choice: locale } = useLocale();
  const system = useApp((state) => state.system);
  const systemName = LOCALES.find((entry) => entry.id === systemLocale())?.name ?? "English";

  const themes: { value: ThemeChoice; label: string }[] = [
    { value: "system", label: s.themes.system },
    { value: "light", label: s.themes.light },
    { value: "dark", label: s.themes.dark },
  ];
  const sizes = ZOOM_LEVELS.map((level) => ({
    value: level,
    label: t.shell.zoom.level(Math.round(level * 100), level === DEFAULT_ZOOM),
  }));
  const languages: { value: LocaleChoice; label: string }[] = [
    { value: "system", label: t.shell.language.system(systemName) },
    ...LOCALES.map((entry) => ({ value: entry.id as LocaleChoice, label: entry.name })),
  ];

  return (
    <Dialog title={s.title} onClose={onClose} width="lg">
      <div className="flex flex-col gap-6">
        <Section title={s.appearance}>
          <Field label={s.theme}>
            <Choices label={s.theme} value={theme} options={themes} onChange={setTheme} />
          </Field>
          <Field label={s.size} hint={s.sizeHint}>
            <Choices<ZoomLevel> label={s.size} value={zoom} options={sizes} onChange={setZoom} />
          </Field>
        </Section>
        <Section title={s.language}>
          <Choices
            label={s.language}
            value={locale}
            options={languages}
            onChange={setLocaleChoice}
          />
        </Section>
        <Section title={s.about}>
          <ul className="text-ink-soft text-sm leading-relaxed" data-selectable>
            {system && <li>{s.botloft(system.daemonVersion)}</li>}
            {system?.claudeVersion && <li>{s.claudeCode(system.claudeVersion)}</li>}
            {system?.account.claude?.email && <li>{system.account.claude.email}</li>}
          </ul>
        </Section>
      </div>
    </Dialog>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">{title}</h3>
      {children}
    </section>
  );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <p className="font-medium text-sm">{label}</p>
      {children}
      {hint && <p className="text-muted text-xs">{hint}</p>}
    </div>
  );
}
