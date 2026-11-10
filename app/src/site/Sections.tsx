// The shorter parts of the site's page (spec 39.1): the strip of facts
// under the opening scene, the three steps to start and the heading every
// section shares.

import type { SiteMessages } from "../i18n/en/site";
import { CLAUDE_SETUP_URL } from "./locales";
import { Mascot } from "./Mascot";

export function Facts({ t }: { t: SiteMessages }) {
  return (
    <ul aria-label={t.facts.label} className="site-facts">
      {t.facts.items.map((fact) => (
        <li key={fact}>{fact}</li>
      ))}
    </ul>
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
    <section id="start" className="site-section site-band">
      <div className="site-wrap">
        <SectionHead title={t.steps.title} />
        <ol className="site-steps">
          {STEPS.map(({ key, color }, index) => (
            <li key={key}>
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
    <div className="mx-auto mb-14 max-w-2xl text-center">
      <h2 className="site-h2">{title}</h2>
      {lead && <p className="site-lead mt-4">{lead}</p>}
    </div>
  );
}
