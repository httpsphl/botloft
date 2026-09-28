// Images the owner attached, as data URLs for thumbnails. What the app sent
// in this session is known already; the rest is read back from the bot's
// folder with `attachments.read` (spec 9.5). The CSP allows `data:` images
// only, so no blob URLs.

import { useEffect, useState } from "react";
import type { Attachment } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

/** Types the webview shows safely as `<img>`. */
const IMAGE_TYPES = new Set(["image/png", "image/jpeg", "image/gif", "image/webp", "image/bmp"]);
/** Bigger images show as a file card: reading them back costs too much. */
const THUMBNAIL_MAX_BYTES = 8 * 1024 * 1024;
const CACHE_MAX = 60;

const cache = new Map<string, string>();

export function isImage(mediaType: string): boolean {
  return IMAGE_TYPES.has(mediaType.toLowerCase());
}

export function dataUrl(mediaType: string, base64: string): string {
  return `data:${mediaType};base64,${base64}`;
}

/** Remembers an image the app has the bytes of. */
export function rememberImage(attachmentId: string, url: string): void {
  cache.delete(attachmentId);
  cache.set(attachmentId, url);
  while (cache.size > CACHE_MAX) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) {
      break;
    }
    cache.delete(oldest);
  }
}

export type ImageSource =
  | { kind: "none" }
  | { kind: "loading" }
  | { kind: "ready"; url: string }
  | { kind: "missing" };

/** The thumbnail of an attachment, or `none` for files that are not images. */
export function useImage(attachment: Attachment): ImageSource {
  const api = useApi();
  const shown = isImage(attachment.mediaType) && attachment.size <= THUMBNAIL_MAX_BYTES;
  const [source, setSource] = useState<ImageSource>(() => {
    const known = cache.get(attachment.id);
    if (known) {
      return { kind: "ready", url: known };
    }
    return shown ? { kind: "loading" } : { kind: "none" };
  });

  useEffect(() => {
    if (!shown || cache.has(attachment.id)) {
      return;
    }
    let alive = true;
    api.call("attachments.read", { attachmentId: attachment.id }).then(
      (file) => {
        const url = dataUrl(file.mediaType, file.data);
        rememberImage(attachment.id, url);
        if (alive) {
          setSource({ kind: "ready", url });
        }
      },
      () => alive && setSource({ kind: "missing" }),
    );
    return () => {
      alive = false;
    };
  }, [api, attachment.id, shown]);

  return source;
}
