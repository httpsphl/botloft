// Builds the setup window (spec 15.7) around the NSIS installer that
// `pnpm bundle` made: the page with Vite, then the botloft-setup crate with
// the installer inside, copied to
// target/release/bundle/setup/Botloft_<version>_x64-setup.exe.
//
//   node scripts/setup.mjs              with the installer inside
//   node scripts/setup.mjs --rehearse   without it: the pages only pretend

import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { root, workspaceVersion } from "./workspace.mjs";

const app = join(root, "app");
const target = process.env.CARGO_TARGET_DIR ?? join(root, "target");
const version = workspaceVersion();
const rehearse = process.argv.includes("--rehearse");
const installer = join(target, "release", "bundle", "nsis", `Botloft_${version}_x64-setup.exe`);
if (!rehearse && !existsSync(installer)) {
  throw new Error(`${installer} is missing: run pnpm bundle first`);
}

execFileSync(
  process.execPath,
  [join(app, "node_modules", "vite", "bin", "vite.js"), "build", "--mode", "setup"],
  {
    cwd: app,
    stdio: "inherit",
  },
);

const env = { ...process.env };
delete env.BOTLOFT_SETUP_PAYLOAD;
if (!rehearse) {
  env.BOTLOFT_SETUP_PAYLOAD = installer;
}
execFileSync(
  "cargo",
  ["build", "--release", "-p", "botloft-setup", "--features", "tauri/custom-protocol"],
  { cwd: root, stdio: "inherit", env },
);

execFileSync(
  process.execPath,
  [join(app, "scripts", "sign.mjs"), join(target, "release", "botloft-setup.exe")],
  { stdio: "inherit" },
);

const out = join(target, "release", "bundle", "setup");
mkdirSync(out, { recursive: true });
const setup = join(out, `Botloft_${version}_x64-setup.exe`);
copyFileSync(join(target, "release", "botloft-setup.exe"), setup);
console.log(`setup${rehearse ? " (rehearsal)" : ""}: ${setup}`);
