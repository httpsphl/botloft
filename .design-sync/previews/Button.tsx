import { Button, icons, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Variants() {
  return (
    <div className="flex flex-wrap items-center gap-2 p-4">
      <Button variant="primary">Create crew</Button>
      <Button variant="secondary">Cancel</Button>
      <Button variant="ghost">Skip for now</Button>
      <Button variant="danger">Delete bot</Button>
    </div>
  );
}

export function WithIcons() {
  return (
    <div className="flex flex-wrap items-center gap-2 p-4">
      <Button variant="primary" icon={icons.Plus}>
        New bot
      </Button>
      <Button icon={icons.Pause}>Pause crew</Button>
      <Button variant="ghost" icon={icons.Settings} label="Settings" />
      <Button variant="ghost" icon={icons.Ellipsis} label="More" />
    </div>
  );
}

export function Sizes() {
  return (
    <div className="flex flex-wrap items-center gap-2 p-4">
      <Button size="sm" icon={icons.RotateCw}>
        Restart
      </Button>
      <Button size="md" icon={icons.RotateCw}>
        Restart
      </Button>
      <Button size="sm" variant="ghost" icon={icons.X} label="Close" />
    </div>
  );
}

export function Disabled() {
  return (
    <div className="flex flex-wrap items-center gap-2 p-4">
      <Button variant="primary" disabled>
        Send
      </Button>
      <Button disabled>Cancel</Button>
    </div>
  );
}

export function DarkTheme() {
  return (
    <div data-theme="dark" className="flex flex-wrap items-center gap-2 bg-canvas p-4 text-ink">
      <Button variant="primary" icon={icons.Plus}>
        New bot
      </Button>
      <Button>Cancel</Button>
      <Button variant="ghost">Skip for now</Button>
      <Button variant="danger">Delete bot</Button>
    </div>
  );
}
