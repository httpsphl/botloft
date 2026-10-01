// Puts this checkout over the Botloft installed on this computer, without
// an installer: builds the app and botloftd for release, swaps both into
// the install folder and has the scheduled task run the new daemon. The
// owner's data stays where it is; the database moves forward to this
// build's migrations, so an older daemon will no longer open it.
//
//   pnpm local            build, swap, reopen the app
//   pnpm local --no-build install what the last build left in target/release
//   pnpm local --restore  put back what the last run replaced

import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(app, "..");
const target = process.env.CARGO_TARGET_DIR ?? join(root, "target");
const restore = process.argv.includes("--restore");
const build = !restore && !process.argv.includes("--no-build");

// The installed copy, never a dev one (CLAUDE.md: BOTLOFT_HOME).
const home = join(process.env.LOCALAPPDATA ?? "", "Botloft");
const env = { ...process.env };
delete env.BOTLOFT_HOME;
const folder = installFolder();
const backup = join(folder, "local-backup");
const files = ["Botloft.exe", "botloftd.exe"];

if (!existsSync(join(folder, "Botloft.exe"))) {
  throw new Error(`no installed Botloft in ${folder}; install it once with the setup first`);
}

if (restore && !files.every((file) => existsSync(join(backup, file)))) {
  throw new Error(`nothing to restore in ${backup}`);
}

if (build) {
  execFileSync("cargo", ["build", "--release", "-p", "botloftd"], { cwd: root, stdio: "inherit" });
  // `tauri build` embeds the UI; a plain `cargo build` would point it at
  // the dev server. Without the bundle config there is no sidecar to need.
  execFileSync("pnpm", ["tauri", "build", "--no-bundle"], {
    cwd: app,
    stdio: "inherit",
    shell: true,
  });
}

closeApp();
if (restore) {
  for (const file of files) {
    copyFileSync(join(backup, file), join(folder, file));
  }
} else {
  mkdirSync(backup, { recursive: true });
  for (const file of files) {
    copyFileSync(join(folder, file), join(backup, file));
    copyFileSync(join(target, "release", file), join(folder, file));
  }
}
// Copies the daemon next to the data, restarts the task and waits for the
// new version to answer (spec 14).
execFileSync(join(folder, "botloftd.exe"), ["--home", home, "service", "install"], {
  env,
  stdio: "inherit",
});
// Through `start`, so the app holds none of this terminal's handles.
execFileSync("cmd", ["/c", "start", '""', `"${join(folder, "Botloft.exe")}"`], {
  env,
  windowsVerbatimArguments: true,
});
console.log(
  restore ? `restored the previous build in ${folder}` : `installed this build in ${folder}`,
);

/** Where the installer put Botloft (hooks.nsh), or the default folder. */
function installFolder() {
  try {
    const out = execFileSync("reg", ["query", "HKCU\\Software\\Botloft\\Botloft", "/ve"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    const path = out.match(/REG_SZ\s+(.+?)\s*$/m)?.[1];
    if (path) {
      return path.replace(/^"|"$/g, "");
    }
  } catch {
    // No key: an install older than the key, in the default folder.
  }
  return home;
}

/** The running app locks its exe; the daemon keeps the bots meanwhile. */
function closeApp() {
  const exe = join(folder, "Botloft.exe").replaceAll("'", "''");
  execFileSync(
    "powershell",
    [
      "-NoProfile",
      "-Command",
      `Get-Process Botloft -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq '${exe}' } | ForEach-Object { Stop-Process -Id $_.Id -Force; $_.WaitForExit(5000) | Out-Null }; exit 0`,
    ],
    { stdio: "inherit" },
  );
}
