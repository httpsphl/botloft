// Sounds and videos of a bot, played in the chat and the files panel
// (spec 8.4, 15.1). They play from an address on the daemon's own server
// (`files.media`), read in pieces as they play, so a long video never
// crosses the connection whole.

import { useCallback, useEffect, useRef, useState } from "react";
import { type BotFile, RpcErrorCode } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi } from "../../store/context";

export type MediaKind = "audio" | "video";

// What the daemon gives a sound or video type (spec 8.4). A card stored
// before it knew some of them says `application/octet-stream`.
const AUDIO = new Set(["mp3", "wav", "m4a", "aac", "ogg", "oga", "opus", "flac", "weba"]);
const VIDEO = new Set(["mp4", "m4v", "webm", "mov"]);

/** Whether a file plays as a sound or a video, by its type or name. */
export function mediaKind(file: Pick<BotFile, "mediaType" | "name">): MediaKind | null {
  const type = file.mediaType.toLowerCase();
  if (type.startsWith("audio/")) {
    return "audio";
  }
  if (type.startsWith("video/")) {
    return "video";
  }
  const dot = file.name.lastIndexOf(".");
  const extension = dot > 0 ? file.name.slice(dot + 1).toLowerCase() : "";
  return AUDIO.has(extension) ? "audio" : VIDEO.has(extension) ? "video" : null;
}

export type MediaLinkState =
  | { kind: "loading" }
  | { kind: "ready"; url: string }
  /** `gone`: the file is no longer there; otherwise it cannot play here. */
  | { kind: "failed"; gone: boolean };

const LINKS_MAX = 200;
const links = new Map<string, string>();

function remember(key: string, url: string): void {
  links.delete(key);
  links.set(key, url);
  while (links.size > LINKS_MAX) {
    const oldest = links.keys().next().value;
    if (oldest === undefined) {
      break;
    }
    links.delete(oldest);
  }
}

/**
 * Where a file of the bot plays from, asked once and kept. `failed()` is for
 * a player that could not play it: a kept link may have died with the
 * daemon, so a new one is asked for once; when that fails too, the file is
 * one this app cannot play.
 */
export function useMediaLink(botId: string, path: string) {
  const api = useApi();
  const key = `${botId}\n${path}`;
  const [state, setState] = useState<MediaLinkState>(() => {
    const known = links.get(key);
    return known ? { kind: "ready", url: known } : { kind: "loading" };
  });
  const [attempt, setAttempt] = useState(0);
  const retried = useRef<string | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: a new attempt asks again
  useEffect(() => {
    const known = links.get(key);
    if (known) {
      setState({ kind: "ready", url: known });
      return;
    }
    let alive = true;
    setState({ kind: "loading" });
    api.call("files.media", { botId, path }).then(
      (link) => {
        remember(key, link.url);
        if (alive) {
          setState({ kind: "ready", url: link.url });
        }
      },
      (failure: unknown) => {
        if (alive) {
          const gone = failure instanceof RpcError && failure.code === RpcErrorCode.notFound;
          setState({ kind: "failed", gone });
        }
      },
    );
    return () => {
      alive = false;
    };
  }, [api, botId, path, key, attempt]);

  const failed = useCallback(() => {
    links.delete(key);
    if (retried.current === key) {
      setState({ kind: "failed", gone: false });
      return;
    }
    retried.current = key;
    setAttempt((count) => count + 1);
  }, [key]);

  return { state, failed };
}

/** One sound or video plays at a time: starting one pauses the last. */
let playing: HTMLMediaElement | null = null;

export function startedPlaying(element: HTMLMediaElement): void {
  if (playing && playing !== element) {
    playing.pause();
  }
  playing = element;
}

/** "1:05" from 65 seconds, "1:02:03" past an hour. */
export function clock(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return "0:00";
  }
  const whole = Math.floor(seconds);
  const hours = Math.floor(whole / 3600);
  const minutes = Math.floor((whole % 3600) / 60);
  const rest = String(whole % 60).padStart(2, "0");
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${rest}` : `${minutes}:${rest}`;
}
