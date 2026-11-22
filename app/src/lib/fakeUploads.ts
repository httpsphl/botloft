// The fake daemon's uploads (spec 42.7): a file sent in pieces from another
// computer, kept in memory, and the path it would have on the server.

import { decodeBytes } from "./base64";
import type { FakeBotloft, Handlers } from "./fake";
import { RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type UploadMethods = Extract<keyof Handlers, `uploads.${string}`>;

export interface FakeUpload {
  name: string;
  size: number;
  bytes: number[];
  /** Where `uploads.finish` said it is, once all of it came. */
  path: string | null;
}

const states = new WeakMap<FakeBotloft, Map<string, FakeUpload>>();

/** What came to the fake daemon so far, by upload id. */
export function fakeUploads(fake: FakeBotloft): Map<string, FakeUpload> {
  let uploads = states.get(fake);
  if (!uploads) {
    uploads = new Map();
    states.set(fake, uploads);
  }
  return uploads;
}

export function uploadHandlers(fake: FakeBotloft): Pick<Handlers, UploadMethods> {
  const uploads = fakeUploads(fake);
  const find = (uploadId: string) => {
    const upload = uploads.get(uploadId);
    if (!upload) {
      throw new RpcError(RpcErrorCode.notFound, `upload ${uploadId}`);
    }
    return upload;
  };
  return {
    "uploads.start": ({ name, size }) => {
      const uploadId = `up${uploads.size + 1}`;
      uploads.set(uploadId, { name, size, bytes: [], path: null });
      return { uploadId };
    },
    "uploads.chunk": ({ uploadId, offset, data }) => {
      const upload = find(uploadId);
      if (offset !== upload.bytes.length) {
        throw new RpcError(RpcErrorCode.conflict, `the upload has ${upload.bytes.length} bytes`);
      }
      // A loop: spreading a whole piece would overflow the stack.
      for (const byte of decodeBytes(data)) {
        upload.bytes.push(byte);
      }
      return null;
    },
    "uploads.finish": ({ uploadId }) => {
      const upload = find(uploadId);
      if (upload.bytes.length !== upload.size) {
        throw new RpcError(RpcErrorCode.conflict, "not all of it came");
      }
      upload.path = `/var/lib/botloft/home/uploads/${uploadId}/${upload.name}`;
      return { path: upload.path };
    },
    "uploads.cancel": ({ uploadId }) => {
      uploads.delete(uploadId);
      return null;
    },
  };
}
