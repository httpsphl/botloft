// The small scenes of the site's cards (spec 39.4), part one: the live
// browser, a request to approve, a task passed between agents and the
// routines. Each is a scene the player moves (player.ts).

import {
  Ban,
  Check,
  Globe,
  Hand,
  Lock,
  MousePointer2,
  RotateCw,
  Send,
  Terminal,
} from "lucide-react";
import type { SiteMessages } from "../../i18n/en/site";
import { Mascot } from "../Mascot";
import { AGENT_COLORS } from "./HeroDemo";

type Demo = SiteMessages["demos"];

export function BrowserDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-browser"
      data-scene
      data-steps="6"
      data-beat="1300"
      data-hold="3200"
    >
      <div className="demo-browser-top">
        <span className="demo-live">{d.browser.live}</span>
        <span className="demo-tab">
          <Globe size={12} />
          {d.browser.tab}
        </span>
      </div>
      <div className="demo-url">
        <RotateCw size={12} />
        <Lock size={11} />
        https://recipes.example/
      </div>
      <div className="demo-page">
        <p className="demo-page-title">{d.browser.page}</p>
        <div className="demo-search">
          <span className="demo-field">
            <span data-at="1" data-type>
              {d.browser.query}
            </span>
          </span>
          <span className="demo-go">{d.browser.search}</span>
        </div>
        <ul className="demo-results">
          {d.browser.results.map((result, index) => (
            <li key={result} data-at={3 + Math.min(index, 1)}>
              <b>{result}</b>
              <span />
            </li>
          ))}
        </ul>
        <span className="demo-cursor">
          <MousePointer2 size={16} />
          <span>Scout</span>
        </span>
      </div>
      <div className="demo-control">
        <Mascot color={AGENT_COLORS.scout} size={20} mood="working" />
        {d.browser.control}
        <span className="demo-btn">
          <Hand size={12} />
          {d.browser.take}
        </span>
      </div>
    </div>
  );
}

export function AskDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-ask"
      data-scene
      data-steps="4"
      data-beat="1400"
      data-hold="3400"
    >
      <div className="demo-card" data-until="2">
        <p className="demo-card-title">
          <span className="demo-badge">
            <Terminal size={14} />
          </span>
          {d.ask.wants}
        </p>
        <code className="demo-command">{d.ask.command}</code>
        <div className="demo-buttons">
          <span className="demo-btn demo-press" data-kind="yes">
            <Check size={13} />
            {d.ask.allow}
          </span>
          <span className="demo-btn" data-kind="no">
            <Ban size={13} />
            {d.ask.deny}
          </span>
        </div>
      </div>
      <p className="demo-settled" data-at="2">
        <Check size={14} />
        {d.ask.allowed}
        <code>{d.ask.command}</code>
      </p>
      <p className="demo-reply" data-at="3" data-type>
        {d.ask.after}
      </p>
    </div>
  );
}

export function HandoffDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-handoff-scene"
      data-scene
      data-steps="4"
      data-beat="1300"
      data-hold="3000"
    >
      <div className="demo-pair">
        <span className="demo-agent">
          <Mascot color={AGENT_COLORS.scout} size={58} mood="awake" moods="0:working,2:awake" />
          <b>Scout</b>
        </span>
        <span className="demo-path">
          <span className="demo-letter">
            <Send size={14} />
          </span>
        </span>
        <span className="demo-agent">
          <Mascot color={AGENT_COLORS.writer} size={58} mood="awake" moods="0:awake,2:working" />
          <b>Writer</b>
        </span>
      </div>
      <p className="demo-task" data-at="2">
        <Send size={13} />
        {d.handoff.task}
      </p>
    </div>
  );
}

export function RoutinesDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-routines"
      data-scene
      data-steps="3"
      data-beat="1500"
      data-hold="3200"
    >
      <ul>
        {d.routines.items.map((routine, index) => (
          <li key={routine.name} data-first={index === 0 || undefined}>
            <span>
              <b>{routine.name}</b>
              <small>{routine.when}</small>
            </span>
            {index === 0 && (
              <>
                <span className="demo-run" data-at="1" data-until="2">
                  <RotateCw size={12} />
                  {d.routines.running}
                </span>
                <span className="demo-done" data-at="2">
                  <Check size={12} />
                  {d.routines.done}
                </span>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
