// Settings, "Appearance": theme, size and motion (spec 15.3).

import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { setTheme, type ThemeChoice, useTheme } from "../../shell/theme";
import { DEFAULT_ZOOM, setZoom, useZoom, ZOOM_LEVELS, type ZoomLevel } from "../../shell/zoom";
import { Choices } from "../../ui/Choices";
import { Field, Section, Toggle } from "./settingsParts";

export function AppearanceSettings() {
  const t = useT();
  const s = t.account.settings;
  const { choice: theme } = useTheme();
  const zoom = useZoom();
  const lessMotion = usePref(prefs.lessMotion);
  const themes: { value: ThemeChoice; label: string }[] = [
    { value: "system", label: s.themes.system },
    { value: "light", label: s.themes.light },
    { value: "dark", label: s.themes.dark },
  ];
  const sizes = ZOOM_LEVELS.map((level) => ({
    value: level,
    label: t.shell.zoom.level(Math.round(level * 100), level === DEFAULT_ZOOM),
  }));

  return (
    <Section title={s.appearance}>
      <Field label={s.theme}>
        <Choices label={s.theme} value={theme} options={themes} onChange={setTheme} />
      </Field>
      <Field label={s.size} hint={s.sizeHint}>
        <Choices<ZoomLevel> label={s.size} value={zoom} options={sizes} onChange={setZoom} />
      </Field>
      <Toggle
        label={s.lessMotion}
        hint={s.lessMotionHint}
        checked={lessMotion}
        onChange={prefs.lessMotion.set}
      />
    </Section>
  );
}
