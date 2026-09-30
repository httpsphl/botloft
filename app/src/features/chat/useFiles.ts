// Files the owner is about to send (spec 9.5), read as base64 when they are
// picked, pasted or dropped. Images keep a data URL for their thumbnail.

import { useCallback, useMemo, useState } from "react";
import { t } from "../../i18n";
import { fileSize } from "../../lib/format";
import { FIELD_LIMITS } from "../../lib/protocol.gen";
import { isImage } from "./images";

/**
 * Most bytes one message carries. Base64 adds a third, and the daemon
 * takes WebSocket messages up to 32 MiB.
 */
export const MESSAGE_FILES_MAX_BYTES = 22 * 1024 * 1024;

export interface PendingFile {
  key: string;
  name: string;
  mediaType: string;
  size: number;
  /** Base64. */
  data: string;
  /** Data URL of an image, for its thumbnail. */
  preview: string | null;
}

let nextKey = 1;

function read(file: File): Promise<PendingFile> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error(t().chat.files.unreadable(file.name)));
    reader.onload = () => {
      const url = String(reader.result);
      const mediaType = file.type || "application/octet-stream";
      resolve({
        key: `file-${nextKey++}`,
        name: file.name || "file",
        mediaType,
        size: file.size,
        data: url.slice(url.indexOf(",") + 1),
        preview: isImage(mediaType) ? url : null,
      });
    };
    reader.readAsDataURL(file);
  });
}

export function useFiles() {
  const [files, setFiles] = useState<PendingFile[]>([]);
  const [error, setError] = useState<string | null>(null);

  const add = useCallback(
    async (picked: File[]) => {
      if (picked.length === 0) {
        return;
      }
      const count = files.length + picked.length;
      const total = [...files, ...picked].reduce((sum, file) => sum + file.size, 0);
      if (count > FIELD_LIMITS.attachments) {
        setError(t().chat.files.tooMany(FIELD_LIMITS.attachments));
        return;
      }
      if (total > MESSAGE_FILES_MAX_BYTES) {
        setError(t().chat.files.tooBig(fileSize(MESSAGE_FILES_MAX_BYTES)));
        return;
      }
      setError(null);
      try {
        const loaded = await Promise.all(picked.map(read));
        setFiles((current) => [...current, ...loaded]);
      } catch (failure) {
        setError(failure instanceof Error ? failure.message : String(failure));
      }
    },
    [files],
  );

  const remove = useCallback((key: string) => {
    setFiles((current) => current.filter((file) => file.key !== key));
  }, []);

  const clear = useCallback(() => {
    setFiles([]);
    setError(null);
  }, []);

  // One object while nothing changes, so the composer can skip renders.
  return useMemo(
    () => ({ files, error, setError, add, remove, clear }),
    [files, error, add, remove, clear],
  );
}

export type Files = ReturnType<typeof useFiles>;
