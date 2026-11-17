// Telling the owner when a bot needs them or finishes (spec 15.1): a
// Windows notification while Botloft is not in front (with the Windows
// sound, if the owner wants sound), the chime while it is, and that bot
// opened when the owner comes back from the notification. "Finished" waits
// for the real end: not while work it handed out is still coming back. A
// question asked or a note left (23.10) needs the owner too.

import { useEffect } from "react";
import { toolTitle } from "../features/chat/toolNames";
import { t } from "../i18n";
import { plainText } from "../lib/plainText";
import type { Bot, BotId, BotState, Question } from "../lib/protocol.gen";
import { SYSTEM } from "../lib/system";
import { type AppState, activityOf } from "../store/app";
import { useAppStore, useHost } from "../store/context";
import { prefs, wantsDone } from "./prefs";
import { playSound } from "./sounds";

/** How long after a notification opening Botloft goes to its bot. */
const FOLLOW_MS = 10 * 60 * 1000;

type Kind = "approval" | "signIn" | "done";

function kindOf(before: BotState | undefined, now: BotState): Kind | null {
  if (before === undefined || before === now) {
    return null;
  }
  if (now === "needs_approval") {
    return "approval";
  }
  if (now === "auth_error") {
    return "signIn";
  }
  return before === "busy" && now === "idle" ? "done" : null;
}

/** A run ended this recently is the turn that just ended: its run comes first. */
const QUIET_MS = 10_000;

/**
 * The turn that just ended was a routine run with nothing new (spec 20.5):
 * there is nothing to tell. The daemon closes the run before the bot goes
 * idle, so its `routine.run` is already in the store.
 */
export function endedQuietly(state: AppState, botId: BotId, now = Date.now()): boolean {
  return Object.values(state.routines).some((routine) => {
    const run = routine.lastRun;
    return (
      routine.botId === botId &&
      run?.nothingNew === true &&
      run.finishedAt !== null &&
      now - run.finishedAt < QUIET_MS
    );
  });
}

/**
 * The bot stopped, but its work is not over (spec 15.1): a task it asked of
 * another agent is still open, or something it left running in the
 * background is. Either brings it back to work when it ends, and the turn
 * after that is the one that finishes.
 */
export function stillWaiting(state: AppState, bot: Bot): boolean {
  return (
    bot.background.length > 0 ||
    Object.values(state.tasks).some(
      (task) => task.requesterBotId === bot.id && task.status === "open",
    )
  );
}

type Words = { title: string; body: string };

/** Most of a question or note shown in its notification. */
const NOTICE_TEXT_MAX = 140;

/** "Scout left you a note", and the note's first words. */
export function questionNotice(bot: Bot, question: Question): Words {
  const words = t().alerts;
  const text = plainText(question.text).replace(/\s+/g, " ").trim();
  return {
    title: question.kind === "note" ? words.tells(bot.name) : words.asks(bot.name),
    body: text.length > NOTICE_TEXT_MAX ? `${text.slice(0, NOTICE_TEXT_MAX - 1)}…` : text,
  };
}

function notice(state: AppState, bot: Bot, kind: Kind): Words {
  const words = t();
  const crew = state.crews[bot.crewId]?.name ?? "";
  switch (kind) {
    case "approval": {
      const activity = activityOf(state, bot.id);
      const tool = activity?.kind === "approval" ? activity.tool : null;
      return {
        title: words.alerts.approval(bot.name),
        body: tool ? toolTitle(tool, words.tools) : words.alerts.crew(crew),
      };
    }
    case "signIn":
      return { title: words.alerts.signIn(bot.name), body: words.alerts.signInBody };
    case "done":
      return { title: words.alerts.done(bot.name), body: words.alerts.crew(crew) };
  }
}

export function useBotAlerts(): void {
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let seen = new Map<BotId, BotState>();
    let last: { bot: BotId; at: number } | null = null;

    let asked = new Set(Object.keys(store.getState().questions));

    /** The chime while Botloft is in front; else the notification. */
    const tell = (bot: BotId, words: Words, chime: "needs" | "done") => {
      if (host.window.inFront()) {
        playSound(chime);
        return;
      }
      const sound = prefs.sound.get();
      last = { bot, at: Date.now() };
      const toast = () => host.notify({ ...words, sound }).catch(() => {});
      // The island shows it instead (spec 31.6), unless the screen is quiet,
      // where Windows keeps its own rule.
      if (SYSTEM === "Windows" && prefs.island.get()) {
        host.quiet().then((quiet) => {
          if (quiet) {
            toast();
          } else if (sound) {
            playSound(chime);
          }
        }, toast);
        return;
      }
      toast();
    };

    const follow = (state: AppState) => {
      const now = new Map(Object.values(state.bots).map((bot) => [bot.id, bot.state]));
      // What was there on (re)connecting is not news.
      const loading = !state.loaded;
      const before = seen;
      const askedBefore = asked;
      seen = now;
      asked = new Set(Object.keys(state.questions));
      if (loading || before.size === 0) {
        return;
      }
      for (const bot of Object.values(state.bots)) {
        const kind = bot.archivedAt === null ? kindOf(before.get(bot.id), bot.state) : null;
        const wanted =
          kind === "done"
            ? wantsDone(bot.id) && !endedQuietly(state, bot.id) && !stillWaiting(state, bot)
            : prefs.notifyNeeds.get();
        if (kind && wanted) {
          tell(bot.id, notice(state, bot, kind), kind === "done" ? "done" : "needs");
        }
      }
      // A question asked or a note left since (spec 23.10).
      if (!prefs.notifyNeeds.get()) {
        return;
      }
      for (const question of Object.values(state.questions)) {
        const bot = state.bots[question.botId];
        if (!askedBefore.has(question.id) && bot && bot.archivedAt === null) {
          tell(bot.id, questionNotice(bot, question), "needs");
        }
      }
    };

    seen = new Map(Object.values(store.getState().bots).map((bot) => [bot.id, bot.state]));
    const unsubscribe = store.subscribe(follow);
    let off: (() => void) | undefined;
    let alive = true;
    host.window
      .onReopened(() => {
        const bot = last && Date.now() - last.at < FOLLOW_MS ? last.bot : null;
        last = null;
        if (bot && store.getState().bots[bot]) {
          store.getState().selectBot(bot);
        }
      })
      .then(
        (unsubscribeReopen) => {
          if (alive) {
            off = unsubscribeReopen;
          } else {
            unsubscribeReopen();
          }
        },
        () => {},
      );
    return () => {
      alive = false;
      unsubscribe();
      off?.();
    };
  }, [host, store]);
}
