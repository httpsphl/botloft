// The end of the site's page: the engines, privacy, the questions, the
// last call to download and the foot (spec 39.1).

import { Code, EyeOff, KeyRound, LockKeyhole, type LucideIcon } from "lucide-react";
import { BOTLOFT_COLOR } from "../features/bots/BotAvatar";
import type { SiteMessages } from "../i18n/en/site";
import { GetButtons, Languages } from "./Controls";
import { REPO_URL, type SiteLocale } from "./locales";
import { Mascot } from "./Mascot";
import { SectionHead } from "./Sections";
import { DOWNLOADS } from "./systems";

export function Engines({ t }: { t: SiteMessages }) {
  const engines = [
    { key: "claude", color: BOTLOFT_COLOR, main: true },
    { key: "others", color: "#8b6bff", main: false },
  ] as const;
  return (
    <section className="site-section site-band">
      <div className="site-wrap">
        <SectionHead title={t.engines.title} lead={t.engines.lead} />
        <div className="mx-auto grid max-w-4xl gap-6 md:grid-cols-2">
          {engines.map(({ key, color, main }) => (
            <div key={key} className="site-card" data-main={main || undefined}>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <span className="site-tag">{t.engines[key].tag}</span>
                  <h3 className="site-card-title mt-3 text-xl">{t.engines[key].title}</h3>
                </div>
                <Mascot color={color} size={48} mood={main ? "working" : "awake"} />
              </div>
              <p className="site-card-body">{t.engines[key].body}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

type PrivacyKey = keyof SiteMessages["privacy"]["items"];

const PRIVACY: readonly { key: PrivacyKey; icon: LucideIcon }[] = [
  { key: "local", icon: EyeOff },
  { key: "sealed", icon: LockKeyhole },
  { key: "keys", icon: KeyRound },
  { key: "code", icon: Code },
];

export function Privacy({ t }: { t: SiteMessages }) {
  return (
    <section id="privacy" className="site-section">
      <div className="site-wrap">
        <SectionHead title={t.privacy.title} lead={t.privacy.lead} />
        <ul className="mx-auto grid max-w-4xl gap-x-10 gap-y-8 sm:grid-cols-2">
          {PRIVACY.map(({ key, icon: Icon }) => (
            <li key={key} className="flex gap-4">
              <span className="site-icon shrink-0">
                <Icon size={20} aria-hidden />
              </span>
              <div>
                <h3 className="site-card-title mt-0">{t.privacy.items[key].title}</h3>
                <p className="site-card-body">{t.privacy.items[key].body}</p>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

export function Faq({ t }: { t: SiteMessages }) {
  const items = Object.entries(t.faq.items);
  return (
    <section id="faq" className="site-section site-band">
      <div className="site-wrap max-w-3xl">
        <SectionHead title={t.faq.title} />
        <div className="site-faq">
          {items.map(([key, item]) => (
            <details key={key}>
              <summary>{item.q}</summary>
              <p>{item.a}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}

const CLOSING_CREW = [
  { color: "#3fe0c5", mood: "awake" },
  { color: "#ffcf3f", mood: "awake" },
  { color: BOTLOFT_COLOR, mood: "working" },
  { color: "#5ec8ff", mood: "awake" },
] as const;

export function Closing({ t }: { t: SiteMessages }) {
  const files = [
    { href: DOWNLOADS.windows, label: t.download.windowsFile },
    { href: DOWNLOADS.macos, label: t.download.macosFile },
    { href: DOWNLOADS.deb, label: t.download.debFile },
    { href: DOWNLOADS.appImage, label: t.download.appImageFile },
  ];
  return (
    <section id="downloads" className="site-section">
      <div className="site-wrap">
        <div className="site-closing">
          <div className="site-crew site-crew-small">
            {CLOSING_CREW.map((mascot) => (
              <Mascot key={mascot.color} {...mascot} size={mascot.mood === "working" ? 72 : 54} />
            ))}
          </div>
          <h2 className="site-h2">{t.closing.title}</h2>
          <p className="site-lead mt-3">{t.closing.body}</p>
          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <GetButtons t={t} />
          </div>
          <nav aria-label={t.download.all} className="site-files">
            {files.map((file) => (
              <a key={file.href} href={file.href}>
                {file.label}
              </a>
            ))}
          </nav>
        </div>
      </div>
    </section>
  );
}

export function Footer({ locale, t }: { locale: SiteLocale; t: SiteMessages }) {
  const links = [
    { href: REPO_URL, label: t.nav.github },
    { href: `${REPO_URL}/releases`, label: t.footer.releases },
    { href: `${REPO_URL}/blob/main/LICENSE`, label: t.footer.license },
    { href: `${REPO_URL}/blob/main/docs/code-signing-policy.md`, label: t.footer.signing },
    { href: `${REPO_URL}/blob/main/SECURITY.md`, label: t.footer.security },
    { href: `${REPO_URL}#privacy-and-security`, label: t.footer.account },
  ];
  return (
    <footer className="site-footer">
      <div className="site-wrap flex flex-col gap-8 md:flex-row md:items-start md:justify-between">
        <div className="max-w-sm">
          <p className="flex items-center gap-2 text-lg font-semibold">
            <Mascot color={BOTLOFT_COLOR} size={26} mood="awake" />
            Botloft
          </p>
          <p className="mt-2 text-muted">{t.footer.tagline}</p>
          <div className="mt-5">
            <Languages locale={locale} label={t.nav.language} />
          </div>
        </div>
        <nav aria-label={t.footer.links} className="site-footer-links">
          {links.map((link) => (
            <a key={link.href} href={link.href}>
              {link.label}
            </a>
          ))}
        </nav>
      </div>
      <div className="site-wrap mt-10 flex flex-col gap-1 border-t border-line pt-6 text-sm text-muted sm:flex-row sm:justify-between">
        <span>© 2026 Phelipe Lorran</span>
        <span>{t.footer.notAffiliated}</span>
      </div>
    </footer>
  );
}
