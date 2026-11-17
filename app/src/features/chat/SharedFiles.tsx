// Files the bot shared with `share_file` (spec 10, 15.3): one card each
// (FileCard.tsx), with the file's picture, player or tile, its name, kind
// and size, and buttons to preview it, open it and save a copy anywhere.

import { useT } from "../../i18n";
import type { ChatItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { FileCard } from "./FileCard";
import { sharedFiles } from "./shared";

export function SharedFiles({ item, botId, bot }: { item: ChatItem; botId: string; bot: string }) {
  const t = useT().files;
  const arrival = useArrival(item.createdAt);
  if (item.body.kind !== "tool") {
    return null;
  }
  const files = sharedFiles(item.body);
  if (files.length === 0) {
    return null;
  }
  return (
    <ul aria-label={t.shared(bot)} className={`flex flex-wrap gap-2 ${arrival}`}>
      {files.map((file) => (
        <li key={file.path} className="max-w-full">
          <FileCard botId={botId} file={file} />
        </li>
      ))}
    </ul>
  );
}
