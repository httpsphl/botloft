// The site's one page (spec 39), the same in every language: the top, the
// sections and the foot. It is drawn once per language when the site is
// built (render.tsx); the page's script only wakes the mascots and picks
// the visitor's download (main.tsx).

import { Code, Download } from "lucide-react";
import heroDark from "../../../docs/assets/hero-dark.png";
import heroLight from "../../../docs/assets/hero-light.png";
import { BOTLOFT_COLOR } from "../features/bots/BotAvatar";
import type { SiteMessages } from "../i18n/en/site";
import { GetButtons, Languages } from "./Controls";
import { CLAUDE_SETUP_URL, messagesOf, REPO_URL, SITE_LOCALES, type SiteLocale } from "./locales";
import { Mascot } from "./Mascot";
import { Closing, Engines, Faq, Footer, Privacy } from "./More";
import { Features, SHOT_HEIGHT, SHOT_WIDTH, Shots, Steps } from "./Sections";
import { MAIN_DOWNLOAD } from "./systems";

/** The crew on top of the hero's window: one at work, the others watching. */
const CREW = [
  { color: "#5ec8ff", size: 60, mood: "awake" },
  { color: "#8b6bff", size: 70, mood: "awake" },
  { color: BOTLOFT_COLOR, size: 92, mood: "working" },
  { color: "#4ade5c", size: 70, mood: "awake" },
  { color: "#ff5fa8", size: 60, mood: "awake" },
] as const;

export function Page({ locale }: { locale: SiteLocale }) {
  const t = messagesOf(locale);
  return (
    <>
      <a href="#content" className="site-skip">
        {t.skip}
      </a>
      <Header locale={locale} t={t} />
      <main id="content">
        <Hero t={t} />
        <Features t={t} />
        <Shots t={t} />
        <Steps t={t} />
        <Engines t={t} />
        <Privacy t={t} />
        <Faq t={t} />
        <Closing t={t} />
      </main>
      <Footer locale={locale} t={t} />
    </>
  );
}

function Header({ locale, t }: { locale: SiteLocale; t: SiteMessages }) {
  const home = SITE_LOCALES.find((each) => each.id === locale)?.path ?? "/";
  return (
    <header className="site-header">
      <div className="site-wrap flex h-16 items-center gap-8">
        <a href={home} className="flex items-center gap-2 text-lg font-semibold tracking-tight">
          <Mascot color={BOTLOFT_COLOR} size={30} mood="awake" />
          Botloft
        </a>
        <nav aria-label={t.nav.label} className="site-nav hidden md:flex">
          <a href="#features">{t.nav.features}</a>
          <a href="#start">{t.nav.how}</a>
          <a href="#privacy">{t.nav.privacy}</a>
          <a href="#faq">{t.nav.faq}</a>
        </nav>
        <div className="ml-auto flex items-center gap-3">
          <Languages locale={locale} label={t.nav.language} />
          <a href={REPO_URL} className="site-nav-link hidden sm:inline">
            {t.nav.github}
          </a>
          <a
            href={MAIN_DOWNLOAD.windows}
            data-download
            className="site-button site-button-small hidden sm:inline-flex"
          >
            <Download size={15} aria-hidden />
            {t.download.short}
          </a>
        </div>
      </div>
    </header>
  );
}

function Hero({ t }: { t: SiteMessages }) {
  return (
    <section className="site-hero">
      <div className="site-wrap text-center">
        <p className="site-badge">{t.hero.badge}</p>
        <h1 className="site-title">{t.hero.title}</h1>
        <p className="site-lead mx-auto">{t.hero.lead}</p>
        <div className="mt-9 flex flex-wrap items-center justify-center gap-3">
          <GetButtons t={t} />
          <a href={REPO_URL} className="site-button site-button-big site-button-quiet">
            <Code size={18} aria-hidden />
            {t.hero.github}
          </a>
        </div>
        <p className="mt-5 text-[15px] text-muted">
          {t.hero.needs}{" "}
          <a href={CLAUDE_SETUP_URL} className="site-link">
            {t.hero.needsLink}
          </a>
        </p>
      </div>
      <div className="site-wrap mt-16">
        <div className="site-stage">
          <div className="site-crew">
            {CREW.map((mascot) => (
              <Mascot key={mascot.color} {...mascot} />
            ))}
          </div>
          <div className="site-window">
            <picture>
              <source media="(prefers-color-scheme: dark)" srcSet={heroDark} />
              <img
                src={heroLight}
                alt={t.hero.shot}
                width={SHOT_WIDTH}
                height={SHOT_HEIGHT}
                fetchPriority="high"
              />
            </picture>
          </div>
        </div>
      </div>
    </section>
  );
}
