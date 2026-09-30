// Release steps for .github/workflows/release.yml (spec 15.5).
//
//   node app/scripts/release.mjs check     the tag matches the workspace version
//   node app/scripts/release.mjs publish   draft GitHub release with the installer,
//                                          its signature and latest.json
//
// latest.json is the feed the in-app updater reads from the latest published
// release. A draft is invisible to it until the owner publishes the release.

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { root, workspaceVersion } from "./workspace.mjs";

const tag = process.env.GITHUB_REF_NAME ?? "";
const repo = process.env.GITHUB_REPOSITORY ?? "";

function check() {
  const version = workspaceVersion();
  if (tag !== `v${version}`) {
    throw new Error(`tag ${tag || "(none)"} does not match the workspace version ${version}`);
  }
  console.log(`releasing ${version}`);
}

function publish() {
  const version = workspaceVersion();
  const dir = join(root, "target", "release", "bundle", "nsis");
  const installer = `Botloft_${version}_x64-setup.exe`;
  const signature = readFileSync(join(dir, `${installer}.sig`), "utf8").trim();
  const feed = {
    version,
    notes: "",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature,
        url: `https://github.com/${repo}/releases/download/${tag}/${installer}`,
      },
    },
  };
  const latest = join(dir, "latest.json");
  writeFileSync(latest, `${JSON.stringify(feed, null, 2)}\n`);
  execFileSync(
    "gh",
    [
      "release",
      "create",
      tag,
      join(dir, installer),
      join(dir, `${installer}.sig`),
      latest,
      "--draft",
      "--title",
      `Botloft ${version}`,
      "--generate-notes",
    ],
    { stdio: "inherit" },
  );
}

const step = process.argv[2];
if (step === "check") {
  check();
} else if (step === "publish") {
  publish();
} else {
  throw new Error("usage: release.mjs check|publish");
}
