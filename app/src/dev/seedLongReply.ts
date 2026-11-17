// Dev only: the kind of long reply an agent sends after digging into
// something, for working on how replies read.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const REPORT = `I went through last night's error log, the deploy history and the payment provider's status page. Short version: **a config change in Friday's deploy made the checkout time out for about 40 minutes**, and it fixed itself when the cache expired. Nothing was charged twice, but 23 orders failed and those customers saw an error.

## What happened

At 21:14 the Friday deploy went out with a new \`timeout_ms\` of \`800\`. The old value was \`8000\`: a zero went missing. Most payments take between 1 and 3 seconds, so almost every checkout after that timed out. The site kept retrying, the payment provider started limiting us, and that is why some errors say "too many requests" instead of "timeout".

At 21:55 the config cache expired and the app read the value again from the environment, which still had the right number, because the deploy only changed the file. From then on everything worked.

## Numbers

| Time | Checkouts | Failed | Main error |
|---|---|---|---|
| 21:00 to 21:14 | 112 | 1 | none |
| 21:14 to 21:55 | 64 | 23 | timeout |
| 21:55 to 23:00 | 140 | 0 | none |

## What I checked

- **Error log** (\`logs/app-2026-10-29.log\`): 23 \`PaymentTimeout\` errors and 9 \`RateLimited\`, all between 21:14 and 21:55.
- **Deploy history**: only one deploy in that window, the Friday one, commit \`4f2a9c1\`.
- **Payment provider**: no incident on their status page last night, so it was on our side.
- **Orders**: none of the 23 failed orders was charged. I checked each one against the provider's export:
  - 19 customers tried again and paid within the hour
  - 4 never came back (their orders are in \`shared/ops/lost-checkouts.csv\`)

## What I'd do now

1. Put \`timeout_ms\` back to \`8000\` in \`config/payments.toml\`, so the next cache refresh doesn't bring the problem back. **This is the urgent one**: the next refresh is tonight at 21:00.
2. Add a check to the deploy that refuses a payment timeout under 2 seconds.
3. Send the 4 customers who didn't come back a short apology with a link to their cart.
4. Ask @deploy to add config values to the Friday checklist.

The change in the deploy was this one line:

\`\`\`diff
- timeout_ms = 8000
+ timeout_ms = 800
\`\`\`

I haven't changed anything yet. Should I go ahead with 1 and 2? For 3 I'd like you to read the email first.`;

export function seedLongReply(fake: FakeBotloft, watcher: BotId): void {
  const { chat } = fake;
  void fake.call("messages.send", {
    botId: watcher,
    body: "What happened with the checkout errors last night? Give me the full picture.",
  });
  chat.finish(chat.tool(watcher, "Read", { summary: "logs/app-2026-10-29.log" }), "4,210 lines");
  chat.finish(chat.tool(watcher, "Grep", { summary: "PaymentTimeout|RateLimited" }), "32 matches");
  chat.finish(chat.tool(watcher, "Bash", { summary: "git log --since=yesterday" }), "1 commit");
  chat.reply(watcher, REPORT);
  chat.turn(watcher);
}
