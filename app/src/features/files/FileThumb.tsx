// The face of a file in the files panel: a picture's own thumbnail, read
// with `files.read` once the row comes into sight (spec 15.1), or the
// colored tile of its kind. A picture too big to read back keeps the tile.

import { useEffect, useRef, useState } from "react";
import type { BotFile } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { dataUrl, isImage, remembered, rememberImage } from "../chat/images";
import { type FileKind, kindIcon } from "./kinds";

/** Bigger pictures keep their tile: reading them back costs too much. */
const THUMB_MAX_BYTES = 8 * 1024 * 1024;

/** True once the element has been on screen (always, without the observer). */
function useSeen<T extends Element>() {
  const ref = useRef<T>(null);
  const [seen, setSeen] = useState(() => typeof IntersectionObserver !== "function");
  useEffect(() => {
    const element = ref.current;
    if (seen || !element) {
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setSeen(true);
          observer.disconnect();
        }
      },
      { rootMargin: "120px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }, [seen]);
  return [ref, seen] as const;
}

/** The picture's data URL once read; null while it is not (or never will be). */
export function useThumb(botId: string, file: BotFile, wanted: boolean): string | null {
  const api = useApi();
  const can = isImage(file.mediaType) && file.size <= THUMB_MAX_BYTES;
  const key = `file:${file.path}:${file.modifiedAt}`;
  const [url, setUrl] = useState(() => (can ? (remembered(key) ?? null) : null));
  useEffect(() => {
    if (!can) {
      setUrl(null);
      return;
    }
    const known = remembered(key);
    if (known) {
      setUrl(known);
      return;
    }
    if (!wanted) {
      return;
    }
    let alive = true;
    api.call("files.read", { botId, path: file.path }).then(
      (read) => {
        const shown = dataUrl(read.mediaType, read.data);
        rememberImage(key, shown);
        if (alive) {
          setUrl(shown);
        }
      },
      () => {
        // The tile stays.
      },
    );
    return () => {
      alive = false;
    };
  }, [api, botId, file.path, key, can, wanted]);
  return url;
}

/** The kind's tile, or the picture itself when it can be read. */
export function FileThumb({
  botId,
  file,
  kind,
  size,
}: {
  botId: string;
  file: BotFile;
  kind: FileKind;
  /** "row": a small square in front of the name; "card": a square that fills its cell. */
  size: "row" | "card";
}) {
  const [ref, seen] = useSeen<HTMLSpanElement>();
  const url = useThumb(botId, file, seen);
  const Icon = kindIcon(kind);
  const card = size === "card";
  return (
    <span
      ref={ref}
      aria-hidden
      className={card ? "file-tile file-tile-card" : "file-tile file-tile-row"}
      data-kind={kind}
      data-picture={url ? "" : undefined}
    >
      {url ? (
        <img src={url} alt="" loading="lazy" decoding="async" />
      ) : (
        <Icon size={card ? 28 : 18} />
      )}
    </span>
  );
}
