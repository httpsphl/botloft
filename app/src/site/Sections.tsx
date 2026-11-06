// The middle of the site's page: what Botloft does, the screenshots and
// the three steps to start (spec 39.1).

import {
  ArrowLeftRight,
  CalendarClock,
  LibraryBig,
  type LucideIcon,
  MessageCircle,
  Power,
  ShieldCheck,
  Smartphone,
  Users,
  WandSparkles,
} from "lucide-react";
import chatBrowser from "../../../docs/assets/chat-browser.png";
import chief from "../../../docs/assets/chief.png";
import routines from "../../../docs/assets/routines.png";
import screens from "../../../docs/assets/screens.png";
import type { SiteMessages } from "../i18n/en/site";
import { CLAUDE_SETUP_URL } from "./locales";
import { Mascot } from "./Mascot";

/** The screenshots' own size, so the page keeps their place while they load. */
export const SHOT_WIDTH = 1800;
export const SHOT_HEIGHT = 1125;

type FeatureKey = keyof SiteMessages["features"]["items"];

const FEATURES: readonly { key: FeatureKey; icon: LucideIcon }[] = [
  { key: "crew", icon: Users },
  { key: "agency", icon: LibraryBig },
  { key: "maker", icon: WandSparkles },
  { key: "chat", icon: MessageCircle },
  { key: "ask", icon: ShieldCheck },
  { key: "handoff", icon: ArrowLeftRight },
  { key: "always", icon: Power },
  { key: "routines", icon: CalendarClock },
  { key: "phone", icon: Smartphone },
];

export function Features({ t }: { t: SiteMessages }) {
  return (
    <section id="features" className="site-section">
      <div className="site-wrap">
        <SectionHead title={t.features.title} lead={t.features.lead} />
        <ul className="site-grid">
          {FEATURES.map(({ key, icon: Icon }) => (
            <li key={key} className="site-card">
              <span className="site-icon">
                <Icon size={20} aria-hidden />
              </span>
              <h3 className="site-card-title">{t.features.items[key].title}</h3>
              <p className="site-card-body">{t.features.items[key].body}</p>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

type ShotKey = keyof SiteMessages["shots"]["items"];

const SHOTS: readonly { key: ShotKey; src: string }[] = [
  { key: "browser", src: chatBrowser },
  { key: "screens", src: screens },
  { key: "chief", src: chief },
  { key: "routines", src: routines },
];

export function Shots({ t }: { t: SiteMessages }) {
  return (
    <section className="site-section site-band">
      <div className="site-wrap">
        <SectionHead title={t.shots.title} />
        <div className="grid gap-6 md:grid-cols-2">
          {SHOTS.map(({ key, src }) => (
            <figure key={key} className="site-shot">
              <img
                src={src}
                alt={t.shots.items[key].alt}
                width={SHOT_WIDTH}
                height={SHOT_HEIGHT}
                loading="lazy"
                decoding="async"
              />
              <figcaption>
                <strong>{t.shots.items[key].title}.</strong> {t.shots.items[key].body}
              </figcaption>
            </figure>
          ))}
        </div>
        <p className="mt-6 text-center text-sm text-muted">{t.shots.note}</p>
      </div>
    </section>
  );
}

type StepKey = keyof SiteMessages["steps"]["items"];

/** Each step with a mascot of its own, in the crew's colors. */
const STEPS: readonly { key: StepKey; color: string }[] = [
  { key: "engine", color: "#5ec8ff" },
  { key: "app", color: "#ff7a59" },
  { key: "crew", color: "#4ade5c" },
];

export function Steps({ t }: { t: SiteMessages }) {
  return (
    <section id="start" className="site-section">
      <div className="site-wrap">
        <SectionHead title={t.steps.title} />
        <ol className="grid gap-6 md:grid-cols-3">
          {STEPS.map(({ key, color }, index) => (
            <li key={key} className="site-card site-step">
              <div className="flex items-center justify-between">
                <span className="site-step-number">{index + 1}</span>
                <Mascot color={color} size={44} mood="awake" />
              </div>
              <h3 className="site-card-title">{t.steps.items[key].title}</h3>
              <p className="site-card-body">
                {t.steps.items[key].body}
                {key === "engine" && (
                  <>
                    {" "}
                    <a href={CLAUDE_SETUP_URL} className="site-link">
                      {t.steps.items.engine.link}
                    </a>
                  </>
                )}
              </p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}

export function SectionHead({ title, lead }: { title: string; lead?: string }) {
  return (
    <div className="mx-auto mb-12 max-w-2xl text-center">
      <h2 className="site-h2">{title}</h2>
      {lead && <p className="site-lead mt-4">{lead}</p>}
    </div>
  );
}
