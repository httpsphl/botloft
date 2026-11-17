// What Botloft does, as cards with a small scene each (spec 39.1, 39.4):
// a wide one for the live browser and the Maker, the others in a grid.

import type { ReactNode } from "react";
import type { SiteMessages } from "../i18n/en/site";
import { MakerDemo, PhoneDemo, SleepDemo } from "./demos/MoreDemos";
import { AskDemo, BrowserDemo, HandoffDemo, RoutinesDemo } from "./demos/WorkDemos";
import { SectionHead } from "./Sections";

type CardKey = Exclude<keyof SiteMessages["cards"], "title" | "lead">;

function Card({
  t,
  id,
  wide = false,
  children,
}: {
  t: SiteMessages;
  id: CardKey;
  wide?: boolean;
  children: ReactNode;
}) {
  return (
    <li className="site-demo-card" data-wide={wide || undefined} data-card={id}>
      <div className="site-demo-stage" aria-hidden>
        {children}
      </div>
      <div className="site-demo-text">
        <h3 className="site-card-title">{t.cards[id].title}</h3>
        <p className="site-card-body">{t.cards[id].body}</p>
      </div>
    </li>
  );
}

export function Cards({ t }: { t: SiteMessages }) {
  const d = t.demos;
  return (
    <section id="features" className="site-section">
      <div className="site-wrap">
        <SectionHead title={t.cards.title} lead={t.cards.lead} />
        <ul className="site-bento">
          <Card t={t} id="browser" wide>
            <BrowserDemo d={d} />
          </Card>
          <Card t={t} id="ask">
            <AskDemo d={d} />
          </Card>
          <Card t={t} id="handoff">
            <HandoffDemo d={d} />
          </Card>
          <Card t={t} id="routines">
            <RoutinesDemo d={d} />
          </Card>
          <Card t={t} id="phone">
            <PhoneDemo d={d} />
          </Card>
          <Card t={t} id="maker" wide>
            <MakerDemo d={d} />
          </Card>
          <Card t={t} id="sleep">
            <SleepDemo d={d} />
          </Card>
        </ul>
      </div>
    </section>
  );
}
