// The daemon's part of Settings (spec 11.2): read when shown and saved on
// each change. A change shows at once and goes back if it cannot be saved.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Settings, SettingsUpdateParams } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

export function useDaemonSettings(): {
  settings: Settings | null;
  change(update: SettingsUpdateParams): void;
} {
  const api = useApi();
  const failed = useT().account.settings.saveFailed;
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    let alive = true;
    api.call("settings.get").then(
      (found) => alive && setSettings(found),
      // Shown without them; the switches wait.
      () => {},
    );
    return () => {
      alive = false;
    };
  }, [api]);

  const change = (update: SettingsUpdateParams) => {
    const before = settings;
    if (before) {
      setSettings({
        startWithWindows: update.startWithWindows ?? before.startWithWindows,
        keepAwake: update.keepAwake ?? before.keepAwake,
        approvalWaitMinutes: update.approvalWaitMinutes ?? before.approvalWaitMinutes,
      });
    }
    attempt(failed, async () => setSettings(await api.call("settings.update", update))).then(
      (saved) => saved || setSettings(before),
    );
  };

  return { settings, change };
}
