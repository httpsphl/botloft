// Dev only: Writer calls Revisão now and then, so the preview shows bots
// calling each other (spec 15.3), ringing and then picked up.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const FIRST_MS = 4000;
const EVERY_MS = 40_000;
const PICK_UP_MS = 6000;
const TIMES = 3;

export function seedCalls(fake: FakeBotloft, bots: { writer: BotId; reviewer: BotId }): void {
  let made = 0;
  const call = () => {
    fake.now = Date.now();
    const { delivery } = fake.conversation.say({
      from: bots.writer,
      to: bots.reviewer,
      kind: "task",
      body: "The draft is in shared/drafts/week-39-report.md. Check the facts and the tone?",
    });
    setTimeout(() => {
      fake.now = Date.now();
      fake.conversation.read(delivery.id);
    }, PICK_UP_MS);
    made += 1;
    if (made < TIMES) {
      setTimeout(call, EVERY_MS);
    }
  };
  setTimeout(call, FIRST_MS);
}
