// `HostTransfers` for tests: downloads kept in memory, and a picked file
// the test sets.

import { decodeBytes, encodeBytes } from "./base64";
import type { HostTransfers, PickedFile } from "./host";

export interface FakeDownload {
  name: string;
  /** Saved where the owner picked, or written to the temporary folder. */
  temp: boolean;
  bytes: number[];
  /** Ended and kept; opened with its program too. */
  kept: boolean;
  opened: boolean;
}

export class FakeTransfers implements HostTransfers {
  readonly downloads: FakeDownload[] = [];
  /** Whether the Save dialog picks a place. */
  saveChosen = true;
  /** What `pick` gives next, with its bytes; null is a cancel. */
  nextPick: { name: string; bytes: Uint8Array } | null = null;
  private readonly picked = new Map<string, Uint8Array>();
  private next = 1;

  private start(name: string, temp: boolean): string {
    this.downloads.push({ name, temp, bytes: [], kept: false, opened: false });
    return String(this.downloads.length - 1);
  }

  save(name: string): Promise<string | null> {
    return Promise.resolve(this.saveChosen ? this.start(name, false) : null);
  }

  temp(name: string): Promise<string> {
    return Promise.resolve(this.start(name, true));
  }

  write(id: string, data: string): Promise<void> {
    const download = this.downloads[Number(id)];
    for (const byte of decodeBytes(data)) {
      download?.bytes.push(byte);
    }
    return Promise.resolve();
  }

  end(id: string, done: { keep: boolean; open: boolean }): Promise<void> {
    const download = this.downloads[Number(id)];
    if (download) {
      download.kept = done.keep;
      download.opened = done.keep && done.open && download.temp;
    }
    return Promise.resolve();
  }

  pick(): Promise<PickedFile | null> {
    if (!this.nextPick) {
      return Promise.resolve(null);
    }
    const id = `pick${this.next++}`;
    this.picked.set(id, this.nextPick.bytes);
    return Promise.resolve({ id, name: this.nextPick.name, size: this.nextPick.bytes.length });
  }

  read(id: string, offset: number, length: number): Promise<string> {
    const bytes = this.picked.get(id) ?? new Uint8Array();
    return Promise.resolve(encodeBytes(bytes.subarray(offset, offset + length)));
  }
}
