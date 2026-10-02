import { BotAvatar, icons, Menu, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const noop = () => {};

export function BotMenu() {
  return (
    <div className="flex items-center gap-2 p-4">
      <BotAvatar color="#4FC382" size={28} />
      <span className="font-semibold text-ink text-sm">Scout</span>
      <Menu
        label="More for Scout"
        icon={icons.Ellipsis}
        items={[
          { label: "Edit", icon: icons.Pencil, onSelect: noop },
          { label: "Restart", icon: icons.RotateCw, onSelect: noop },
          { label: "Pause", icon: icons.Pause, onSelect: noop },
          { label: "Archive", icon: icons.Archive, onSelect: noop },
          { label: "Delete", icon: icons.Trash2, danger: true, onSelect: noop },
        ]}
      />
    </div>
  );
}
