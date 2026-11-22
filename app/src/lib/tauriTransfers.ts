// `HostTransfers` over the app's own commands (src-tauri/src/transfer.rs).

import { invoke } from "@tauri-apps/api/core";
import type { HostTransfers, PickedFile } from "./host";

export const tauriTransfers: HostTransfers = {
  save: (name) => invoke<string | null>("download_save", { name }),
  temp: (name) => invoke<string>("download_temp", { name }),
  write: (id, data) => invoke<void>("download_write", { id, data }),
  end: (id, { keep, open }) => invoke<void>("download_end", { id, open, keep }),
  pick: (title, kind, extensions) =>
    invoke<PickedFile | null>("upload_pick", { title, kind, extensions }),
  read: (id, offset, length) => invoke<string>("upload_read", { id, offset, length }),
};
