// Release steps for .github/workflows/release.yml (spec 15.5, 15.7).
//
//   node app/scripts/release.mjs check     the tag matches the workspace version
//   node app/scripts/release.mjs stage     the release files, with their final names,
//                                          in target/release/bundle/release
//   node app/scripts/release.mjs publish   stage, then a draft GitHub release with them
//
// A release carries two installers: the setup window people download
// (`-setup.exe`, from `pnpm bundle:setup`) and the NSIS installer the in-app
// updater runs (`-update.exe`, from `pnpm bundle`, with its signature).
// latest.json, the feed the updater reads from the latest published release,
// points to the second. A draft is invisible to it until the owner publishes
// the release.

import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

/** Copies the release files under their final names and writes latest.json. */
function stage() {
  const version = workspaceVersion();
  const bundle = join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "release", "bundle");
  const built = `Botloft_${version}_x64-setup.exe`;
  const setup = `Botloft_${version}_x64-setup.exe`;
  const update = `Botloft_${version}_x64-update.exe`;
  const out = join(bundle, "release");
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });

  copyFileSync(join(bundle, "setup", built), join(out, setup));
  copyFileSync(join(bundle, "nsis", built), join(out, update));
  copyFileSync(join(bundle, "nsis", `${built}.sig`), join(out, `${update}.sig`));
  const signature = readFileSync(join(out, `${update}.sig`), "utf8").trim();
  const feed = {
    version,
    notes: "",
    pub_date: new Date().toISOString(),
    platforms: {
      "windows-x86_64": {
        signature,
        url: `https://github.com/${repo}/releases/download/${tag}/${update}`,
      },
    },
  };
  writeFileSync(join(out, "latest.json"), `${JSON.stringify(feed, null, 2)}\n`);
  const files = [setup, update, `${update}.sig`, "latest.json"].map((name) => join(out, name));
  console.log(`staged in ${out}:\n  ${files.join("\n  ")}`);
  return { version, files };
}

function publish() {
  const { version, files } = stage();
  execFileSync(
    "gh",
    [
      "release",
      "create",
      tag,
      ...files,
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
} else if (step === "stage") {
  stage();
} else if (step === "publish") {
  publish();
} else {
  throw new Error("usage: release.mjs check|stage|publish");
}
