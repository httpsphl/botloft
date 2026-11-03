// Settings, the Maker (spec 36.2, 36.4, 36.5): show it on the left, let it
// build without asking, which asks for a confirmation first, and bring it
// the Claude Code history or the claude.ai export.

import { FileArchive, History } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { prefs, usePref } from "../../shell/prefs";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Section, Toggle } from "../account/settingsParts";
import { useDaemonSettings } from "../account/useDaemonSettings";
import { ClaudeAiImport } from "./ClaudeAiImport";
import { HistoryImport } from "./HistoryImport";

export function MakerSettings() {
  const t = useT();
  const s = t.maker.settings;
  const [importing, setImporting] = useState<"code" | "ai" | null>(null);
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
      <div className="flex flex-wrap gap-2 py-2">
        <Button icon={History} onClick={() => setImporting("code")}>
          {t.maker.history.open}
        </Button>
        <Button icon={FileArchive} onClick={() => setImporting("ai")}>
          {t.maker.claudeAi.open}
        </Button>
      </div>
      {importing === "code" && <HistoryImport onClose={() => setImporting(null)} />}
      {importing === "ai" && <ClaudeAiImport onClose={() => setImporting(null)} />}
    </Section>
  );
}
