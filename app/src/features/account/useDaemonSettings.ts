// The daemon's part of Settings (spec 11.2), kept in the app store: read
// on each connection and saved on each change. A change shows at once and
// goes back if it cannot be saved.

import { useT } from "../../i18n";
import type { Settings, SettingsUpdateParams } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { attempt } from "../../ui/toast";

export function useDaemonSettings(): {
  settings: Settings | null;
  change(update: SettingsUpdateParams): void;
} {
  const api = useApi();
  const failed = useT().account.settings.saveFailed;
  const settings = useApp((state) => state.settings);
  const putSettings = useApp((state) => state.putSettings);

  const change = (update: SettingsUpdateParams) => {
    const before = settings;
    if (before) {
      putSettings({
        startWithWindows: update.startWithWindows ?? before.startWithWindows,
        keepAwake: update.keepAwake ?? before.keepAwake,
        approvalWaitMinutes: update.approvalWaitMinutes ?? before.approvalWaitMinutes,
      });
    }
    attempt(failed, async () => putSettings(await api.call("settings.update", update))).then(
      (saved) => saved || putSettings(before),
    );
  };

  return { settings, change };
}
