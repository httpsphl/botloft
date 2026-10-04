// The fake daemon's lessons (spec 21.13): teaching a bot a task with its
// browser in the owner's hands. `step` plays the daemon reading what the
// owner did; `browser.teach` starts and ends a lesson as the daemon does.

import type { FakeBotloft, Handlers } from "./fake";
import type { BotId, LessonStep } from "./protocol.gen";

export class FakeLesson {
  constructor(private readonly fake: FakeBotloft) {}

  /** The owner did something that means `step`, while teaching. */
  step(botId: BotId, step: Partial<LessonStep> & Pick<LessonStep, "kind" | "label">): void {
    const lesson = this.fake.browser.state(botId).lesson;
    if (lesson) {
      this.fake.browser.set(botId, {
        lesson: [...lesson, { role: null, secret: false, ...step }],
      });
    }
  }

  handlers(): Pick<Handlers, "browser.teach"> {
    return {
      "browser.teach": ({ botId, on }) => {
        this.fake.browser.held(botId);
        const state = this.fake.browser.state(botId);
        if (on) {
          const url = state.url?.split(/[?#]/)[0] ?? null;
          const steps: LessonStep[] = url
            ? [{ kind: "open", label: url, role: null, secret: false }]
            : [];
          this.fake.browser.set(botId, { lesson: steps });
          return steps;
        }
        this.fake.browser.set(botId, { lesson: null });
        return state.lesson ?? [];
      },
    };
  }
}
