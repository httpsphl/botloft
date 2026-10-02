// A soft sound when a bot shares a file in the chat the owner is looking at
// (spec 15.1). Only for what arrives after the chat opened: older items,
// and those loaded from further up, stay quiet.

import { useEffect, useRef } from "react";
import type { ChatItem } from "../../lib/protocol.gen";
import { playSound } from "../../shell/sounds";
import { useHost } from "../../store/context";
import { isSharedFiles } from "./shared";

export function useSharedSound(items: ChatItem[], since: number): void {
  const host = useHost();
  const played = useRef(new Set<string>());

  useEffect(() => {
    const fresh = items.filter(
      (item) => item.updatedAt > since && !played.current.has(item.id) && isSharedFiles(item),
    );
    if (fresh.length === 0) {
      return;
    }
    for (const item of fresh) {
      played.current.add(item.id);
    }
    if (host.window.inFront()) {
      playSound("file");
    }
  }, [items, since, host]);
}
