// Release steps for .github/workflows/release.yml (spec 15.5, 15.7).
//
//   node app/scripts/release.mjs check            the tag matches the workspace version
//   node app/scripts/release.mjs stage <system>   this system's release files, with their
//                                                 final names, in target/release/bundle/release,
//                                                 and <system>.json saying what they are
//   node app/scripts/release.mjs publish          a draft GitHub release with what every
//                                                 system staged, and latest.json for all
//
// Each system builds on its own runner: windows, linux and macos. Windows
// carries the setup window people download (`-setup.exe`, from `pnpm
// bundle:setup`) and the NSIS installer the in-app updater runs
// (`-update.exe`). Linux carries an AppImage and a .deb; macOS a .dmg and
// the .app.tar.gz the updater installs. Each updater file comes with its
// signature. latest.json, the feed the updater reads from the latest
// published release, names one file per system and installer; a draft is
// invisible to it until the owner publishes the release. Between `stage`
// and `publish`, each runner attests where its files come from; the
// draft's notes open with their SHA-256 and how to check that attestation.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { root, workspaceVersion } from "./workspace.mjs";

const tag = process.env.GITHUB_REF_NAME ?? "";
const repo = process.env.GITHUB_REPOSITORY ?? "";
const bundle = join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "release", "bundle");
const staged = join(bundle, "release");
const SYSTEMS = ["windows", "linux", "macos", "server-x86_64", "server-aarch64"];

function check() {
  const version = workspaceVersion();
  if (tag !== `v${version}`) {
    throw new Error(`tag ${tag || "(none)"} does not match the workspace version ${version}`);
  }
  console.log(`releasing ${version}`);
}

/** The one file in `bundle/<dir>` whose name ends with `suffix`. */
function built(dir, suffix) {
  const found = readdirSync(join(bundle, dir)).filter((name) => name.endsWith(suffix));
  if (found.length !== 1) {
    throw new Error(`expected one *${suffix} in bundle/${dir}, found ${found.length}`);
  }
  return join(bundle, dir, found[0]);
}

/** What each system ships: [final name, built file, updater key or null]. */
function outputs(system, version) {
  if (system === "windows") {
    const exe = `Botloft_${version}_x64-setup.exe`;
    return [
      [`Botloft_${version}_x64-setup.exe`, join(bundle, "setup", exe), null],
      [`Botloft_${version}_x64-update.exe`, join(bundle, "nsis", exe), "windows-x86_64"],
    ];
  }
  if (system === "linux") {
    return [
      [
        `Botloft_${version}_amd64.AppImage`,
        built("appimage", ".AppImage"),
        "linux-x86_64-appimage",
      ],
      [`Botloft_${version}_amd64.deb`, built("deb", ".deb"), "linux-x86_64-deb"],
    ];
  }
  // The daemon alone for a Linux server, with its SHA-256 for the installer
  // to check, and the installer itself once (spec 42.3).
  if (system.startsWith("server-")) {
    const arch = system.slice("server-".length);
    const archive = `botloftd-linux-${arch}.tar.gz`;
    const files = [[archive, join(bundle, "server", archive), null, { sha256: true }]];
    if (arch === "x86_64") {
      files.push(["install.sh", join(root, "deploy", "server", "install.sh"), null]);
    }
    return files;
  }
  if (system === "macos") {
    const arch = process.arch === "arm64" ? "aarch64" : "x86_64";
    return [
      [`Botloft_${version}_${arch}.dmg`, built("dmg", ".dmg"), null],
      [`Botloft_${version}_${arch}.app.tar.gz`, built("macos", ".app.tar.gz"), `darwin-${arch}`],
    ];
  }
  throw new Error(`usage: release.mjs stage ${SYSTEMS.join("|")}`);
}

/**
 * Copies this system's files under their final names, with the updater's
 * signatures, and writes <system>.json for `publish`.
 */
function stage(system) {
  const version = workspaceVersion();
  mkdirSync(staged, { recursive: true });
  const manifest = { files: [], updater: {} };
  for (const [name, from, key, extra] of outputs(system, version)) {
    copyFileSync(from, join(staged, name));
    manifest.files.push(name);
    if (extra?.sha256) {
      writeFileSync(join(staged, `${name}.sha256`), `${sha256(join(staged, name))}  ${name}\n`);
      manifest.files.push(`${name}.sha256`);
    }
    if (key) {
      copyFileSync(`${from}.sig`, join(staged, `${name}.sig`));
      manifest.files.push(`${name}.sig`);
      const signature = readFileSync(`${from}.sig`, "utf8").trim();
      manifest.updater[key] = { file: name, signature };
    }
  }
  writeFileSync(join(staged, `${system}.json`), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`staged ${system} in ${staged}`);
  for (const name of manifest.files.filter(isDownload)) {
    console.log(`  ${sha256(join(staged, name))}  ${name}`);
  }
}

/** A file people download, not a signature or a checksum beside one. */
function isDownload(name) {
  return !name.endsWith(".sig") && !name.endsWith(".sha256");
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

/** The manifests every system staged, in a fixed order. */
function manifests() {
  return SYSTEMS.filter((system) => existsSync(join(staged, `${system}.json`))).map((system) => ({
    system,
    ...JSON.parse(readFileSync(join(staged, `${system}.json`), "utf8")),
  }));
}

/** The start of the release notes: how to check a download, and how to install off Windows. */
function notes(all) {
  const downloads = all.flatMap(({ files }) => files.filter(isDownload));
  const rows = downloads
    .map((name) => `| \`${name}\` | \`${sha256(join(staged, name))}\` |`)
    .join("\n");
  const setup = downloads.find((name) => name.endsWith("-setup.exe")) ?? downloads[0];
  return `### Verify your download

| File | SHA-256 |
|---|---|
${rows}

Every file was built by GitHub Actions from this repository, which attested where it comes from. To check a download with the [GitHub CLI](https://cli.github.com/):

\`\`\`
gh attestation verify ${setup} --repo ${repo}
\`\`\`

How releases are made, and what Botloft changes on your computer: [Code signing policy](https://github.com/${repo}/blob/main/docs/code-signing-policy.md).

### Linux and macOS (preview)

New in this release and not tried on many computers yet: please [open an issue](https://github.com/${repo}/issues) for anything that does not work.

- **Linux:** the \`.deb\` for Debian and Ubuntu, or the \`.AppImage\` for any other distribution (make it executable first).
- **macOS (Apple Silicon):** Apple has not notarized the app yet. Open the \`.dmg\`, drag Botloft to Applications and open it once; then allow it in System Settings → Privacy & Security → Open Anyway.
${serverNotes(downloads)}`;
}

/** How to put Botloft on a Linux server, when this release has it (spec 42.3). */
function serverNotes(downloads) {
  if (!downloads.includes("install.sh")) {
    return "";
  }
  return `
### On a Linux server (preview)

Botloft can run on your own Linux server (x86_64 or arm64), with nobody at its screen. As root, or with sudo:

\`\`\`
curl -fsSL https://github.com/${repo}/releases/latest/download/install.sh | sh
\`\`\`

It installs the daemon as a service, with a user of its own, and Claude Code for the agents, then says how to sign in to Claude and to connect the app (\`sudo botloftd pair\`).

With Docker instead, the image \`ghcr.io/${repo}\` has the daemon, Claude Code and Chromium (\`:slim\` without a browser); see [deploy/docker](https://github.com/${repo}/tree/main/deploy/docker).
`;
}

function publish() {
  const version = workspaceVersion();
  const all = manifests();
  if (all.length === 0) {
    throw new Error("run `release.mjs stage <system>` first: nothing is staged");
  }
  const platforms = {};
  for (const { updater } of all) {
    for (const [key, { file, signature }] of Object.entries(updater)) {
      platforms[key] = {
        signature,
        url: `https://github.com/${repo}/releases/download/${tag}/${file}`,
      };
    }
  }
  const feed = { version, notes: "", pub_date: new Date().toISOString(), platforms };
  writeFileSync(join(staged, "latest.json"), `${JSON.stringify(feed, null, 2)}\n`);
  const files = [...all.flatMap(({ files }) => files), "latest.json"].map((name) =>
    join(staged, name),
  );
  const missing = files.filter((file) => !existsSync(file));
  if (missing.length > 0) {
    throw new Error(`staged files are missing:\n  ${missing.join("\n  ")}`);
  }
  console.log(`publishing ${all.map(({ system }) => system).join(", ")}`);
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
      notes(all),
      "--generate-notes",
    ],
    { stdio: "inherit" },
  );
}

const [step, system] = process.argv.slice(2);
if (step === "check") {
  check();
} else if (step === "stage") {
  stage(system);
} else if (step === "publish") {
  publish();
} else {
  throw new Error("usage: release.mjs check|stage <system>|publish");
}
