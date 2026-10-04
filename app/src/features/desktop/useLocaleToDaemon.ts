// The owner's language, told to the daemon when it changes, for the notice
// it shows on screen while a bot uses the real mouse and keyboard (spec
// 24.7). The language at connection goes with `session.hello`.

import { useEffect, useRef } from "react";
import { useLocale } from "../../i18n";
import { useApi } from "../../store/context";

export function useLocaleToDaemon(): void {
  const api = useApi();
  const { locale } = useLocale();
  const told = useRef(locale);
  useEffect(() => {
    if (told.current === locale) {
      return;
    }
    told.current = locale;
    // An older daemon does not know the method: nothing to tell.
    api.call("session.setLocale", { locale }).catch(() => {});
  }, [api, locale]);
}
