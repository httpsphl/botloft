// Release steps for .github/workflows/release.yml (spec 15.5, 15.7).
//
//   node app/scripts/release.mjs check     the tag matches the workspace version
//   node app/scripts/release.mjs stage     the release files, with their final names,
//                                          in target/release/bundle/release
//   node app/scripts/release.mjs publish   a draft GitHub release with what `stage` left
//
// A release carries two installers: the setup window people download
// (`-setup.exe`, from `pnpm bundle:setup`) and the NSIS installer the in-app
// updater runs (`-update.exe`, from `pnpm bundle`, with its signature).
// latest.json, the feed the updater reads from the latest published release,
// points to the second. A draft is invisible to it until the owner publishes
// the release. Between `stage` and `publish`, the workflow attests where the
// installers come from; the draft's notes open with their SHA-256 and how to
// check that attestation.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { root, workspaceVersion } from "./workspace.mjs";

const tag = process.env.GITHUB_REF_NAME ?? "";
const repo = process.env.GITHUB_REPOSITORY ?? "";
const bundle = join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "release", "bundle");
const staged = join(bundle, "release");

function names(version) {
  return {
    setup: `Botloft_${version}_x64-setup.exe`,
    update: `Botloft_${version}_x64-update.exe`,
  };
}

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
  const built = `Botloft_${version}_x64-setup.exe`;
  const { setup, update } = names(version);
  rmSync(staged, { recursive: true, force: true });
  mkdirSync(staged, { recursive: true });

  copyFileSync(join(bundle, "setup", built), join(staged, setup));
  copyFileSync(join(bundle, "nsis", built), join(staged, update));
  copyFileSync(join(bundle, "nsis", `${built}.sig`), join(staged, `${update}.sig`));
  const signature = readFileSync(join(staged, `${update}.sig`), "utf8").trim();
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
  writeFileSync(join(staged, "latest.json"), `${JSON.stringify(feed, null, 2)}\n`);
  console.log(`staged in ${staged}`);
  for (const name of [setup, update]) {
    console.log(`  ${sha256(join(staged, name))}  ${name}`);
  }
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

/** The start of the release notes: how to check a download. */
function verifyNotes(version) {
  const { setup, update } = names(version);
  const rows = [setup, update]
    .map((name) => `| \`${name}\` | \`${sha256(join(staged, name))}\` |`)
    .join("\n");
  return `### Verify your download

| File | SHA-256 |
|---|---|
${rows}

Both installers were built by GitHub Actions from this repository, which attested where they come from. To check a download with the [GitHub CLI](https://cli.github.com/):

\`\`\`
gh attestation verify ${setup} --repo ${repo}
\`\`\`

How releases are made, and what Botloft changes on your computer: [Code signing policy](https://github.com/${repo}/blob/main/docs/code-signing-policy.md).
`;
}

function publish() {
  const version = workspaceVersion();
  const { setup, update } = names(version);
  const files = [setup, update, `${update}.sig`, "latest.json"].map((name) => join(staged, name));
  const missing = files.filter((file) => !existsSync(file));
  if (missing.length > 0) {
    throw new Error(`run \`release.mjs stage\` first; missing:\n  ${missing.join("\n  ")}`);
  }
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
      // GitHub puts these notes before the ones it generates.
      "--notes",
      verifyNotes(version),
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
