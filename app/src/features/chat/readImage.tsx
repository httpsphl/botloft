// The picture a bot is looking at: a `Read` of an image shows it small on
// its tool line and bigger when the line opens (spec 15.3). It is read with
// `files.read`, so only images in the bot's folders show; any other path
// keeps the plain line.

import { useEffect, useState } from "react";
import type { ToolItem } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { dataUrl, type ImageSource, remembered, rememberImage } from "./images";

const IMAGE_FILE = /\.(png|jpe?g|gif|webp|bmp)$/i;
const FILE_PATH = /"file_path"\s*:\s*("(?:[^"\\]|\\.)*")/;

/** The image a `Read` call opens, or null for any other call or file. */
export function readImagePath(tool: ToolItem): string | null {
  if (tool.name !== "Read") {
    return null;
  }
  // The input may be cut short, so the path is found, not parsed whole.
  const found = FILE_PATH.exec(tool.input)?.[1];
  if (!found) {
    return null;
  }
  try {
    const path: unknown = JSON.parse(found);
    return typeof path === "string" && IMAGE_FILE.test(path) ? path : null;
  } catch {
    return null;
  }
}

/** The image of a `Read` call, as it is now in the bot's folder. */
export function useReadImage(botId: string, tool: ToolItem): ImageSource {
  const api = useApi();
  const path = tool.status === "failed" ? null : readImagePath(tool);
  const key = `read:${botId}:${tool.toolUseId}`;
  const [source, setSource] = useState<ImageSource>(() => {
    const known = remembered(key);
    if (known) {
      return { kind: "ready", url: known };
    }
    return path ? { kind: "loading" } : { kind: "none" };
  });

  useEffect(() => {
    if (!path || remembered(key)) {
      return;
    }
    let alive = true;
    api.call("files.read", { botId, path }).then(
      (read) => {
        const url = dataUrl(read.mediaType, read.data);
        rememberImage(key, url);
        if (alive) {
          setSource({ kind: "ready", url });
        }
      },
      () => alive && setSource({ kind: "missing" }),
    );
    return () => {
      alive = false;
    };
  }, [api, botId, key, path]);

  return source;
}

/** The image small, beside the call's words. */
export function ReadThumb({ source, name }: { source: ImageSource; name: string }) {
  if (source.kind !== "ready") {
    return null;
  }
  return (
    <img
      src={source.url}
      alt={name}
      className="size-5 shrink-0 rounded border border-line bg-sunken object-cover"
    />
  );
}

/** The image as large as the opened line allows. */
export function ReadPicture({ source, name }: { source: ImageSource; name: string }) {
  if (source.kind !== "ready") {
    return null;
  }
  return (
    <img
      src={source.url}
      alt={name}
      className="block max-h-64 max-w-full self-start rounded-lg border border-line bg-sunken object-contain"
    />
  );
}
