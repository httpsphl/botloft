import { BotAvatar, BotStateBadge, ChiefBadge, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = { name: "Research" };

export function Badge() {
  return (
    <div className="p-4">
      <ChiefBadge crew={crew} />
    </div>
  );
}

export function InBotHeader() {
  return (
    <div className="flex items-center gap-3 p-4">
      <BotAvatar color="#FF7A59" size={36} />
      <div className="flex flex-col gap-0.5">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-base text-ink">Chief</span>
          <ChiefBadge crew={crew} />
        </div>
        <BotStateBadge bot={{ state: "busy", paused: false }} compact />
      </div>
    </div>
  );
}

export function Compact() {
  return (
    <div className="flex items-center gap-2 p-4">
      <BotAvatar color="#FF7A59" size={20} />
      <span className="font-medium text-ink text-sm">Chief</span>
      <ChiefBadge crew={crew} compact />
    </div>
  );
}
