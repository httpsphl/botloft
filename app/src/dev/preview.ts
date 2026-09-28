// Dev only: `pnpm dev` opened in a plain browser runs the app against the
// in-memory fake daemon, for UI work without Tauri or botloftd.
// `?empty` starts with no crews, `?stopped` with the daemon stopped.
// `?live=<port>` talks to a real dev daemon instead; its owner token is read
// from sessionStorage "botloft.devToken", set by hand, never from the URL.

import type { Connect } from "../features/onboarding/link";
import { type Client, connect, rpcUrl } from "../lib/client";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import type { Host } from "../lib/host";
import { PROTOCOL_VERSION } from "../lib/protocol.gen";

export function previewProps(search: string): { host: Host; connect: Connect } {
  const params = new URLSearchParams(search);
  const live = Number(params.get("live"));
  if (live > 0) {
    return liveProps(live);
  }
  const fake = new FakeBotloft();
  const host = new FakeHost();
  if (params.has("stopped")) {
    host.status = {
      state: "stopped",
      port: 45710,
      home: "C:\\Users\\owner\\AppData\\Local\\Botloft",
    };
  }
  if (!params.has("empty")) {
    seed(fake);
  }
  return { host, connect: () => fake as Client };
}

function liveProps(port: number): { host: Host; connect: Connect } {
  const host = new FakeHost();
  host.status = { state: "running", port, version: "dev", protocol: PROTOCOL_VERSION };
  host.token =
    sessionStorage.getItem("botloft.devToken") ??
    new Error('Set sessionStorage "botloft.devToken" to the dev daemon\'s owner token.');
  const client = { name: "botloft-preview", version: "dev" };
  return {
    host,
    connect: (daemonPort, token) => connect({ url: rpcUrl(daemonPort), token, client }),
  };
}

function seed(fake: FakeBotloft): void {
  const research = fake.addCrew("Research");
  const scout = fake.addBot(research.id, "Scout", "Finds sources and summarizes them");
  const writer = fake.addBot(research.id, "Writer", "Turns notes into a weekly report");
  const reviewer = fake.addBot(
    research.id,
    "Revisão",
    "Checks facts and tone before anything ships",
  );
  const analyst = fake.addBot(research.id, "Analyst", "Crunches the numbers behind each claim");
  fake.setBotState(scout.id, "idle", 3);
  fake.setBotState(writer.id, "busy", 2);
  fake.setBotState(reviewer.id, "needs_approval", 5);
  fake.setBotState(analyst.id, "rate_limited", 1);
  fake.terminals.output(scout.id, SCOUT_SCREEN);
  fake.terminals.output(reviewer.id, PROMPT_SCREEN);
  const ops = fake.addCrew("Ops");
  const deploy = fake.addBot(ops.id, "Deploy", "Ships the site on Fridays");
  const watcher = fake.addBot(ops.id, "Watcher", "Keeps an eye on the error log");
  void fake.call("crews.setPaused", { crewId: ops.id, paused: true });
  fake.setBotState(deploy.id, "offline");
  fake.setBotState(watcher.id, "offline");
}

const ESC = String.fromCharCode(27);
const NEWLINE = String.fromCharCode(13, 10);
const dim = (text: string) => `${ESC}[2m${text}${ESC}[0m`;
const accent = (text: string) => `${ESC}[38;5;209m${text}${ESC}[0m`;
const bold = (text: string) => `${ESC}[1m${text}${ESC}[0m`;

const SCOUT_SCREEN = [
  `${bold("scout")} ${dim("· research")}`,
  "",
  `${accent(">")} Summarize the three newest papers in shared/inbox`,
  "",
  `${accent("●")} Reading shared/inbox/2026-09-27-retrieval.pdf`,
  `${accent("●")} Reading shared/inbox/2026-09-26-agents.pdf`,
  `${accent("●")} Wrote shared/summaries/week-39.md ${dim("(42 lines)")}`,
  "",
  "Done. The summary is in shared/summaries/week-39.md.",
  "",
  `${accent(">")} `,
].join(NEWLINE);

const PROMPT_SCREEN = [
  `${bold("revisao")} ${dim("· research")}`,
  "",
  `${accent("●")} Checking the claims in shared/summaries/week-39.md`,
  "",
  `${ESC}[33mAllow this command?${ESC}[0m  npm run check-links`,
  "  1. Yes",
  "  2. Yes, and don't ask again for this command",
  "  3. No",
  "",
].join(NEWLINE);
