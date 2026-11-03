// A page to measure what working mascots cost (`?flame`, dev only): a row
// of small mascots, as in the list of bots, and one large, as in the chat
// header, all working. `?flame&count=12` sets how many small ones.

import { BotAvatar } from "../features/bots/BotAvatar";

const COLORS = ["#ff7a59", "#5ec8ff", "#8b6bff", "#4ade5c", "#ff5fa8", "#3fe0c5", "#ffcf3f"];

const count = Number(new URLSearchParams(window.location.search).get("count")) || 8;

export function FlameBench() {
  return (
    <div style={{ padding: 32, display: "grid", gap: 24, background: "#111", minHeight: "100vh" }}>
      <BotAvatar color={COLORS[0] as string} size={96} mood="working" />
      <div style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
        {Array.from({ length: count }, (_, index) => (
          <BotAvatar
            // biome-ignore lint/suspicious/noArrayIndexKey: a fixed row
            key={index}
            color={COLORS[index % COLORS.length] as string}
            size={28}
            mood="working"
          />
        ))}
      </div>
    </div>
  );
}
