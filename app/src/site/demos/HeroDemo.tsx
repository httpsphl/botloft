// The opening scene (spec 39.4): a Botloft window with a crew at work.
// The owner asks Scout for sources; Scout asks to use a website, browses,
// answers and hands the work to Writer, whose mascot starts working. The
// player (player.ts) moves it one step per beat; the built page shows its end.

import {
  Activity,
  AlarmClock,
  ArrowUp,
  Ban,
  Check,
  ChevronRight,
  Clock,
  Globe,
  Hand,
  House,
  MessageCircleQuestion,
  Minus,
  Paperclip,
  Search,
  Send,
  Square,
  X,
} from "lucide-react";
import { BOTLOFT_COLOR } from "../../features/bots/BotAvatar";
import type { SiteMessages } from "../../i18n/en/site";
import { Mascot } from "../Mascot";

export const AGENT_COLORS = {
  scout: "#ff7a59",
  writer: "#5ec8ff",
  analyst: "#4ade5c",
  designer: "#ff5fa8",
  clerk: "#3fe0c5",
  planner: "#ffcf3f",
  revisao: "#8b6bff",
} as const;

type Demo = SiteMessages["demos"];
type State = "idle" | "working" | "approval";

const STATE_ICON = { idle: Clock, working: Activity, approval: Hand } as const;

/** A state as the app shows it: an icon and a word in the state's color. */
export function StateLabel({ state, d }: { state: State; d: Demo }) {
  const Icon = STATE_ICON[state];
  const word = state === "approval" ? d.states.approval : d.states[state];
  return (
    <span className="demo-state" data-state={state}>
      <Icon size={12} aria-hidden />
      {word}
    </span>
  );
}

/** A label that changes on a step: `from` until `at`, then `to`. */
function StateSwitch({ from, to, at, d }: { from: State; to: State; at: number; d: Demo }) {
  return (
    <>
      <span data-until={at}>
        <StateLabel state={from} d={d} />
      </span>
      <span data-at={at}>
        <StateLabel state={to} d={d} />
      </span>
    </>
  );
}

/** The step each part of the scene shows on. */
const STEP = { ask: 1, card: 2, allowed: 3, browsed: 4, reply: 5, handoff: 6 } as const;

export function HeroDemo({ t }: { t: SiteMessages }) {
  const d = t.demos;
  return (
    <figure className="demo-window demo-hero" aria-label={t.hero.demo}>
      <div
        className="demo-scene"
        data-scene
        data-steps={STEP.handoff + 1}
        data-beat="1700"
        data-hold="5200"
      >
        <TitleBar crew={d.window.crew} />
        <div className="demo-body" aria-hidden>
          <Rail />
          <aside className="demo-side">
            <p className="demo-crew">
              {d.window.crew}
              <span>{d.window.agents}</span>
            </p>
            <ul>
              <li className="demo-row" data-selected>
                <Mascot
                  color={AGENT_COLORS.scout}
                  size={30}
                  mood="awake"
                  moods={`0:awake,${STEP.card}:working,${STEP.handoff}:awake`}
                />
                <span className="demo-row-text">
                  <b>Scout</b>
                  <span data-until={STEP.card}>
                    <StateLabel state="idle" d={d} />
                  </span>
                  <span data-at={STEP.card} data-until={STEP.handoff}>
                    <StateLabel state="working" d={d} />
                  </span>
                  <span data-at={STEP.handoff}>
                    <StateLabel state="idle" d={d} />
                  </span>
                </span>
              </li>
              <li className="demo-row">
                <Mascot
                  color={AGENT_COLORS.writer}
                  size={30}
                  mood="awake"
                  moods={`0:awake,${STEP.handoff}:working`}
                />
                <span className="demo-row-text">
                  <b>Writer</b>
                  <StateSwitch from="idle" to="working" at={STEP.handoff} d={d} />
                </span>
              </li>
              <li className="demo-row">
                <Mascot color={AGENT_COLORS.analyst} size={30} mood="working" />
                <span className="demo-row-text">
                  <b>Analyst</b>
                  <StateLabel state="working" d={d} />
                </span>
              </li>
              <li className="demo-row">
                <Mascot color={AGENT_COLORS.revisao} size={30} mood="waiting" />
                <span className="demo-row-text">
                  <b>Revisão</b>
                  <StateLabel state="approval" d={d} />
                </span>
              </li>
            </ul>
            <span className="demo-ember" />
          </aside>
          <section className="demo-chat">
            <header className="demo-chat-head">
              <Mascot
                color={AGENT_COLORS.scout}
                size={38}
                mood="awake"
                moods={`0:awake,${STEP.card}:working,${STEP.handoff}:awake`}
              />
              <span>
                <b>Scout</b> <code>@scout</code>
                <small>{d.roles.scout}</small>
              </span>
            </header>
            <div className="demo-log">
              <p className="demo-own" data-at={STEP.ask}>
                {d.hero.ask}
              </p>
              <div className="demo-card" data-at={STEP.card} data-until={STEP.allowed}>
                <p className="demo-card-title">
                  <span className="demo-badge">
                    <Globe size={14} />
                  </span>
                  {d.hero.wants} <code>{d.hero.site}</code>
                </p>
                <div className="demo-buttons">
                  <span className="demo-btn" data-kind="yes">
                    <Check size={13} />
                    {d.hero.allow}
                  </span>
                  <span className="demo-btn">
                    <Clock size={13} />
                    {d.hero.once}
                  </span>
                  <span className="demo-btn" data-kind="no">
                    <Ban size={13} />
                    {d.hero.deny}
                  </span>
                </div>
              </div>
              <p className="demo-settled" data-at={STEP.allowed}>
                <Check size={14} />
                {d.hero.allowed}
              </p>
              <p className="demo-quiet" data-at={STEP.browsed}>
                {d.hero.steps}
                <ChevronRight size={14} />
              </p>
              <p className="demo-reply" data-at={STEP.reply} data-type>
                {d.hero.reply}
              </p>
              <p className="demo-handoff" data-at={STEP.handoff}>
                <Send size={14} />
                <span>
                  <b>{d.hero.sent}</b> {d.hero.task}
                </span>
              </p>
            </div>
            <div className="demo-composer">
              <span>{d.window.message}</span>
              <Paperclip size={15} />
              <span className="demo-send">
                <ArrowUp size={14} />
              </span>
            </div>
          </section>
        </div>
      </div>
    </figure>
  );
}

/** The window's top, as on Windows: the app, where you are, and its buttons. */
export function TitleBar({ crew }: { crew: string }) {
  return (
    <div className="demo-titlebar" aria-hidden>
      <Mascot color={BOTLOFT_COLOR} size={16} mood="awake" />
      <b>Botloft</b>
      <ChevronRight size={13} />
      <span>{crew}</span>
      <ChevronRight size={13} />
      <span>Scout</span>
      <span className="demo-controls">
        <Minus size={14} />
        <Square size={11} />
        <X size={14} />
      </span>
    </div>
  );
}

/** The app's narrow left column: home, search, questions, routines. */
function Rail() {
  return (
    <nav className="demo-rail">
      <House size={17} />
      <Search size={17} />
      <MessageCircleQuestion size={17} />
      <AlarmClock size={17} />
    </nav>
  );
}
