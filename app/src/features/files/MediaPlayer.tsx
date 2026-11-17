// A sound or video of the bot's, where the chat or the files panel shows it
// (spec 15.1). It plays from the daemon's own address (`files.media`), in
// pieces, so a video of any size plays and can jump ahead.

import { LoaderCircle } from "lucide-react";
import { useT } from "../../i18n";
import type { BotFile } from "../../lib/protocol.gen";
import { AudioPlayer } from "./AudioPlayer";
import { type MediaKind, startedPlaying, useMediaLink } from "./media";

export function MediaPlayer({
  botId,
  file,
  kind,
  fill = false,
}: {
  botId: string;
  file: BotFile;
  kind: MediaKind;
  /** In the files panel: a video takes the room it has. */
  fill?: boolean;
}) {
  const t = useT().files;
  const { state, failed } = useMediaLink(botId, file.path);
  switch (state.kind) {
    case "loading":
      return (
        <p role="status" className="flex items-center gap-2 px-3 py-3 text-muted text-xs">
          <LoaderCircle aria-hidden size={14} className="animate-spin" />
          {t.media.loading}
        </p>
      );
    case "failed":
      return (
        <p role="alert" className="px-3 py-3 text-ink-soft text-xs">
          {state.gone ? t.preview.gone : t.media.failed}
        </p>
      );
    case "ready":
      return kind === "audio" ? (
        <AudioPlayer src={state.url} name={file.name} onFail={failed} />
      ) : (
        // biome-ignore lint/a11y/useMediaCaption: the bot's own videos have no captions
        <video
          src={state.url}
          controls
          controlsList="nodownload"
          preload="metadata"
          playsInline
          aria-label={file.name}
          onPlay={(event) => startedPlaying(event.currentTarget)}
          onError={failed}
          className={`block w-full bg-black ${fill ? "max-h-full" : "max-h-80"}`}
        />
      );
  }
}
