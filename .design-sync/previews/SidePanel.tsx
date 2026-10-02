import { BotAvatar, Button, icons, setLocaleChoice, SidePanel } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

// SidePanel sits at the right edge of a row, beside the main area.
export function FilesBesideChat() {
  return (
    <div className="flex w-full border border-line bg-canvas" style={{ height: 360 }}>
      <main className="flex min-w-0 flex-1 flex-col gap-3 p-4">
        <div className="flex items-center gap-2">
          <BotAvatar color="#A48BFF" size={28} />
          <span className="font-semibold text-ink text-sm">Writer</span>
        </div>
        <p className="max-w-sm rounded-xl bg-panel px-3 py-2 text-ink text-sm">
          The draft is in report.md. I kept Scout's three best sources and cut the rest.
        </p>
      </main>
      <SidePanel label="Files" name="preview-files" defaultWidth={300}>
        <header className="flex h-12 items-center justify-between border-line border-b pr-2 pl-4">
          <span className="font-semibold text-ink text-sm">Files</span>
          <Button variant="ghost" size="sm" icon={icons.X} label="Close" />
        </header>
        <ul className="flex flex-col gap-0.5 p-2 text-sm">
          {["report.md", "sources.md", "notes/interviews.md", "charts/usage.png"].map((file) => (
            <li key={file} className="flex items-center gap-2 rounded-lg px-2 py-1.5 text-ink-soft hover:bg-sunken">
              <icons.FileText size={14} />
              {file}
            </li>
          ))}
        </ul>
      </SidePanel>
    </div>
  );
}
