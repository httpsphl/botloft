// Telling the owner when the folder of a deleted bot or crew could not go
// to the Recycle Bin (spec 7.6): the daemon moves it after answering, so
// the news comes as a notification.

import { useEffect } from "react";
import { t } from "../i18n";
import { useApi } from "../store/context";
import { notifyError } from "../ui/toast";

export function useFolderNotices(): void {
  const api = useApi();
  useEffect(
    () =>
      api.subscribe((event) => {
        if (event.name === "folder.recycled" && event.params.error !== null) {
          notifyError(t().shell.recycleFailed(event.params.path), event.params.error);
        }
      }),
    [api],
  );
}
