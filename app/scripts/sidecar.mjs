// Builds botloftd for release and puts it where `tauri build` looks for the
// sidecar: src-tauri/binaries/botloftd-<target triple>.exe (spec 15.4).
// The installer then ships it next to the app as botloftd.exe.

import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");
const root = join(app, "..");
const target = process.env.CARGO_TARGET_DIR ?? join(root, "target");
const exe = process.platform === "win32" ? ".exe" : "";

const rustc = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
const triple = rustc.match(/^host: (\S+)$/m)?.[1];
if (!triple) {
  throw new Error("rustc -vV did not print the host triple");
}

execFileSync("cargo", ["build", "--release", "-p", "botloftd"], { cwd: root, stdio: "inherit" });
const binaries = join(app, "src-tauri", "binaries");
mkdirSync(binaries, { recursive: true });
const sidecar = join(binaries, `botloftd-${triple}${exe}`);
copyFileSync(join(target, "release", `botloftd${exe}`), sidecar);
console.log(`sidecar: ${sidecar}`);
