import { BotAvatar, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = [
  { name: "Chief", color: "#FF7A59" },
  { name: "Scout", color: "#4FC382" },
  { name: "Writer", color: "#6EA6FF" },
  { name: "Editor", color: "#F0B24A" },
  { name: "Ops", color: "#C084FC" },
];

export function Colors() {
  return (
    <div className="flex items-end gap-5 p-4">
      {crew.map((bot) => (
        <div key={bot.name} className="flex flex-col items-center gap-1.5">
          <BotAvatar color={bot.color} size={48} />
          <span className="font-medium text-ink-soft text-xs">{bot.name}</span>
        </div>
      ))}
    </div>
  );
}

export function Moods() {
  const moods = ["idle", "working", "waiting", "tired", "sleeping"] as const;
  return (
    <div className="flex items-end gap-5 p-4">
      {moods.map((mood) => (
        <div key={mood} className="flex flex-col items-center gap-1.5">
          <BotAvatar color="#4FC382" size={48} mood={mood} still />
          <span className="text-muted text-xs">{mood}</span>
        </div>
      ))}
    </div>
  );
}

export function Sizes() {
  return (
    <div className="flex items-end gap-4 p-4">
      {[20, 28, 40, 64].map((size) => (
        <BotAvatar key={size} color="#6EA6FF" size={size} />
      ))}
    </div>
  );
}

export function Framed() {
  return (
    <div className="flex items-center gap-3 p-4">
      <BotAvatar color="#FF7A59" size={48} framed />
      <div>
        <p className="font-semibold text-ink text-sm">Botloft</p>
        <p className="text-muted text-xs">The app's own mark, on the black square of its icon.</p>
      </div>
    </div>
  );
}

export function DarkTheme() {
  return (
    <div data-theme="dark" className="flex items-end gap-5 bg-canvas p-4">
      {crew.map((bot) => (
        <BotAvatar key={bot.name} color={bot.color} size={40} />
      ))}
    </div>
  );
}
