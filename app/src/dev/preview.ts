// Dev only: `pnpm dev` opened in a plain browser runs the app against the
// in-memory fake daemon, for UI work without Tauri or botloftd.
// `?empty` starts with no crews, `?stopped` with the daemon stopped.

import type { Connect } from "../features/onboarding/link";
import type { Client } from "../lib/client";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import type { Host } from "../lib/host";

export function previewProps(search: string): { host: Host; connect: Connect } {
  const params = new URLSearchParams(search);
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
  const ops = fake.addCrew("Ops");
  const deploy = fake.addBot(ops.id, "Deploy", "Ships the site on Fridays");
  const watcher = fake.addBot(ops.id, "Watcher", "Keeps an eye on the error log");
  void fake.call("crews.setPaused", { crewId: ops.id, paused: true });
  fake.setBotState(deploy.id, "offline");
  fake.setBotState(watcher.id, "offline");
}
