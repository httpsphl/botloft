// The repository root and the one version everything ships with: the
// workspace's, in Cargo.toml (spec 15.5).

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function workspaceVersion() {
  const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
  const version = cargo.match(/\[workspace\.package\][^[]*?\nversion = "([^"]+)"/)?.[1];
  if (!version) {
    throw new Error("Cargo.toml has no [workspace.package] version");
  }
  return version;
}
