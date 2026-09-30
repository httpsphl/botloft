// Signs one Windows binary with the release certificate (spec 15.5).
//
//   node scripts/sign.mjs <file>
//
// Tauri runs it for everything that goes into the installer
// (`bundle.windows.signCommand`): the app, the daemon, the NSIS plugins, the
// uninstaller and the installer itself. setup.mjs runs it for the setup
// window, which Tauri does not build. The certificate lives in the Windows
// certificate store of the machine that builds (a cloud key shows up there
// once its client is signed in), and is picked by its SHA-1 thumbprint:
//
//   BOTLOFT_SIGN_THUMBPRINT   the certificate's thumbprint; without it nothing
//                             is signed, as in dev builds and in releases
//                             made before the certificate exists
//   BOTLOFT_SIGN_TIMESTAMP    RFC 3161 timestamp server (a public one by default)
//   BOTLOFT_SIGNTOOL          signtool.exe, when it is not in the Windows SDK

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";

const file = process.argv[2];
if (!file) {
  throw new Error("usage: sign.mjs <file>");
}
const thumbprint = (process.env.BOTLOFT_SIGN_THUMBPRINT ?? "").replace(/\s/g, "");
if (!thumbprint) {
  console.log(`not signing ${basename(file)}: BOTLOFT_SIGN_THUMBPRINT is not set`);
  process.exit(0);
}

/** signtool.exe from the newest Windows SDK on this machine. */
function signtool() {
  if (process.env.BOTLOFT_SIGNTOOL) {
    return process.env.BOTLOFT_SIGNTOOL;
  }
  const kits = join(process.env["ProgramFiles(x86)"] ?? "", "Windows Kits", "10", "bin");
  const versions = existsSync(kits)
    ? readdirSync(kits)
        .filter((name) => /^10\./.test(name))
        .sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
    : [];
  for (const version of versions) {
    const tool = join(kits, version, "x64", "signtool.exe");
    if (existsSync(tool)) {
      return tool;
    }
  }
  throw new Error(`signtool.exe not found under ${kits}; set BOTLOFT_SIGNTOOL`);
}

const tool = signtool();
const timestamp = process.env.BOTLOFT_SIGN_TIMESTAMP || "http://timestamp.digicert.com";
execFileSync(
  tool,
  [
    "sign",
    "/sha1",
    thumbprint,
    "/fd",
    "sha256",
    "/tr",
    timestamp,
    "/td",
    "sha256",
    "/d",
    "Botloft",
    "/du",
    "https://github.com/httpsphl/botloft",
    file,
  ],
  { stdio: "inherit" },
);
// A signature Windows does not trust would only show up on the owner's PC.
execFileSync(tool, ["verify", "/pa", file], { stdio: "inherit" });
console.log(`signed ${basename(file)}`);
