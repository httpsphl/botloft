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
import { seedBrowser } from "./seedBrowser";
import { seedCalls } from "./seedCalls";
import { seedChats } from "./seedChats";
import { seedChief } from "./seedChief";
import { seedContext } from "./seedContext";
import { seedHands } from "./seedHands";
import { activeTab, drawPage } from "./seedPage";
import { seedQuestions } from "./seedQuestions";
import { seedRoutines } from "./seedRoutines";
import { seedScreens } from "./seedScreens";

export function previewProps(search: string): { host: Host; connect: Connect } {
  const params = new URLSearchParams(search);
  const live = Number(params.get("live"));
  if (live > 0) {
    return liveProps(live);
  }
  const fake = new FakeBotloft();
  const host = previewHost();
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
  const host = previewHost();
  host.status = {
    state: "running",
    port,
    version: "dev",
    protocol: PROTOCOL_VERSION,
    outdated: false,
  };
  host.token =
    sessionStorage.getItem("botloft.devToken") ??
    new Error('Set sessionStorage "botloft.devToken" to the dev daemon\'s owner token.');
  const client = { name: "botloft-preview", version: "dev" };
  return {
    host,
    connect: (daemonPort, token) => connect({ url: rpcUrl(daemonPort), token, client }),
  };
}

/** A FakeHost whose zoom shows in a plain browser, like the app's does. */
function previewHost(): FakeHost {
  const host = new FakeHost();
  host.setZoom = (factor) => {
    host.zoom = factor;
    document.documentElement.style.zoom = String(factor);
    return Promise.resolve();
  };
  // A plain browser has no folder picker: every pick gives this folder.
  host.nextFolder = "D:\\Projects\\Bakery site";
  return host;
}

function seed(fake: FakeBotloft): void {
  const now = Date.now();
  fake.system = {
    ...fake.system,
    usage: {
      status: "allowed",
      resetsAt: now + 2 * 3600_000,
      observedAt: now - 5 * 60_000,
      windows: [
        { name: "five_hour", utilization: 0.34, resetsAt: now + 2 * 3600_000 },
        { name: "seven_day", utilization: 0.62, resetsAt: now + 3 * 86_400_000 },
      ],
    },
  };
  const research = fake.addCrew("Research");
  const scout = fake.addBot(research.id, "Scout", "Finds sources and summarizes them");
  const writer = fake.addBot(research.id, "Writer", "Turns notes into a weekly report");
  const reviewer = fake.addBot(
    research.id,
    "Revisão",
    "Checks facts and tone before anything ships",
  );
  const analyst = fake.addBot(research.id, "Analyst", "Crunches the numbers behind each claim");
  const planner = fake.addBot(research.id, "Planner", "Plans changes before touching the code");
  void fake.call("bots.setPermissionMode", { botId: planner.id, mode: "plan" });
  void fake.call("bots.setModel", { botId: analyst.id, model: "haiku" });
  // What Claude Code reported: the plan's default, and the chosen Haiku.
  for (const bot of [scout, writer, reviewer, planner]) {
    fake.bot(bot.id).modelInUse = "claude-opus-5-5";
  }
  fake.bot(analyst.id).modelInUse = "claude-haiku-4-5-20251001";
  fake.setBotState(scout.id, "idle", 3);
  fake.setBotState(writer.id, "busy", 2);
  fake.setBotState(reviewer.id, "needs_approval", 5);
  fake.setBotState(analyst.id, "rate_limited", 1);
  fake.setBotState(planner.id, "needs_approval", 2);
  seedChats(fake, {
    scout: scout.id,
    writer: writer.id,
    reviewer: reviewer.id,
    analyst: analyst.id,
    planner: planner.id,
  });
  seedContext(fake, {
    scout: scout.id,
    writer: writer.id,
    reviewer: reviewer.id,
    analyst: analyst.id,
    planner: planner.id,
  });
  seedRoutines(fake, { scout: scout.id, analyst: analyst.id });
  seedQuestions(fake, { scout: scout.id, analyst: analyst.id });
  seedBrowser(fake, scout.id);
  // Analyst hit the limit in the middle of a search: its browser rests.
  fake.browser.open(analyst.id, "https://stats.example/flour-prices", "Flour prices · Stats");
  fake.browser.paint(analyst.id, (size) => drawPage(activeTab(fake, analyst.id), size));
  fake.browser.rest(analyst.id);
  seedScreens(fake, research.id);
  seedHands(fake, research.id);
  seedCalls(fake, { writer: writer.id, reviewer: reviewer.id });
  const ops = fake.addCrew("Ops");
  const deploy = fake.addBot(ops.id, "Deploy", "Ships the site on Fridays");
  const watcher = fake.addBot(ops.id, "Watcher", "Keeps an eye on the error log");
  void fake.call("crews.setPaused", { crewId: ops.id, paused: true });
  fake.setBotState(deploy.id, "offline");
  fake.setBotState(watcher.id, "offline");
  seedChief(fake);
}
