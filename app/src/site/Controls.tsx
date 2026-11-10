// The controls the site repeats: the language switch and the main
// download button (spec 39.2, 39.3).

import { ArrowRight, Download } from "lucide-react";
import type { SiteMessages } from "../i18n/en/site";
import { SITE_LOCALES, type SiteLocale } from "./locales";
import { MAIN_DOWNLOAD, type System } from "./systems";

/** The three languages, each a page of its own. */
export function Languages({ locale, label }: { locale: SiteLocale; label: string }) {
  return (
    <nav aria-label={label} className="site-languages">
      {SITE_LOCALES.map((each) => (
        <a
          key={each.id}
          href={each.path}
          hrefLang={each.id}
          lang={each.id}
          title={each.name}
          aria-current={each.id === locale ? "page" : undefined}
          data-locale={each.id}
        >
          {each.short}
        </a>
      ))}
    </nav>
  );
}

const GET_LABELS: Record<System, keyof SiteMessages["download"]> = {
  windows: "windows",
  macos: "macos",
  linux: "linux",
  phone: "phone",
};

/**
 * The main download button, one for each system. The built page shows
 * Windows's; the script shows the visitor's own instead (main.tsx).
 */
export function GetButtons({ t }: { t: SiteMessages }) {
  return (
    <>
      {(Object.keys(GET_LABELS) as System[]).map((system) => (
        <a
          key={system}
          href={MAIN_DOWNLOAD[system]}
          data-system={system}
          hidden={system !== "windows"}
          className="site-button site-button-big"
        >
          {system === "phone" ? (
            <ArrowRight size={18} aria-hidden />
          ) : (
            <Download size={18} aria-hidden />
          )}
          {t.download[GET_LABELS[system]]}
        </a>
      ))}
    </>
  );
}
