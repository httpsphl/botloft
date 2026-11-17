// The small scenes of the site's cards (spec 39.4), part two: a request
// reaching the phone, the Maker's plan and idle agents waking for work.

import { Ban, Bell, Check, Sparkles } from "lucide-react";
import { MAKER_COLOR } from "../../features/bots/BotAvatar";
import type { SiteMessages } from "../../i18n/en/site";
import { Mascot } from "../Mascot";
import { AGENT_COLORS } from "./HeroDemo";

type Demo = SiteMessages["demos"];

export function PhoneDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-phone"
      data-scene
      data-steps="4"
      data-beat="1400"
      data-hold="3200"
    >
      <div className="demo-phone-frame">
        <span className="demo-phone-time">{d.phone.time}</span>
        <div className="demo-notice" data-at="1">
          <Bell size={13} />
          <span>
            <b>{d.phone.notice}</b>
            <small>{d.phone.noticeBody}</small>
          </span>
        </div>
        <div className="demo-phone-buttons" data-at="2" data-until="3">
          <span className="demo-btn" data-kind="yes">
            <Check size={12} />
            {d.phone.allow}
          </span>
          <span className="demo-btn" data-kind="no">
            <Ban size={12} />
            {d.phone.deny}
          </span>
        </div>
        <p className="demo-settled" data-at="3">
          <Check size={13} />
          {d.phone.allowed}
        </p>
      </div>
    </div>
  );
}

export function MakerDemo({ d }: { d: Demo }) {
  const last = d.maker.items.length + 2;
  return (
    <div
      className="demo-scene demo-maker"
      data-scene
      data-steps={last + 1}
      data-beat="1100"
      data-hold="3600"
    >
      <div className="demo-maker-who">
        <Mascot
          color={MAKER_COLOR}
          size={84}
          mood="awake"
          moods={`0:awake,1:working,${last}:awake`}
        />
      </div>
      <div className="demo-maker-talk">
        <p className="demo-own" data-at="1">
          {d.maker.ask}
        </p>
        <div className="demo-plan" data-at="2">
          <p className="demo-plan-title">
            <Sparkles size={14} />
            {d.maker.plan}
          </p>
          <ul>
            {d.maker.items.map((item, index) => (
              <li key={item} data-at={2 + index}>
                <Check size={13} />
                {item}
              </li>
            ))}
          </ul>
          <div className="demo-buttons" data-until={last}>
            <span className="demo-btn" data-kind="yes">
              {d.maker.build}
            </span>
            <span className="demo-btn">{d.maker.change}</span>
          </div>
          <p className="demo-settled" data-at={last}>
            <Check size={14} />
            {d.maker.built}
          </p>
        </div>
      </div>
    </div>
  );
}

export function SleepDemo({ d }: { d: Demo }) {
  return (
    <div
      className="demo-scene demo-sleep"
      data-scene
      data-steps="4"
      data-beat="1400"
      data-hold="3200"
    >
      <div className="demo-trio">
        <span className="demo-agent">
          <Mascot color={AGENT_COLORS.analyst} size={50} mood="sleeping" />
          <small>{d.states.sleeping}</small>
        </span>
        <span className="demo-agent demo-waker">
          <span className="demo-ping" data-at="1" data-until="3">
            {d.sleep.message}
          </span>
          <Mascot
            color={AGENT_COLORS.designer}
            size={64}
            mood="sleeping"
            moods="0:sleeping,2:working"
          />
          <small data-until="2">{d.states.sleeping}</small>
          <small data-at="2" className="demo-awake">
            {d.states.working}
          </small>
        </span>
        <span className="demo-agent">
          <Mascot color={AGENT_COLORS.planner} size={50} mood="working" />
          <small className="demo-awake">{d.states.working}</small>
        </span>
      </div>
    </div>
  );
}
