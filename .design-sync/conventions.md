# Building with Botloft

Botloft is a desktop app where a crew of Claude Code bots works together. Each bot is a small flame
mascot in its own color. The look is quiet and warm: neutral surfaces, one orange accent, IBM Plex.

## Setup

- No provider is needed. Link `styles.css` and load `_ds_bundle.js`; components are on `window.Botloft`.
- **Theme:** light by default. Put `data-theme="dark"` on a wrapper (or `<html>`) and every token
  switches. Give the page `bg-canvas text-ink` so the background follows the theme.
- **Language:** the components' own words (state labels, Close, Cancel) follow the browser's
  language. Call `Botloft.setLocaleChoice("en")` once before rendering to pin English
  (`"pt-BR"` and `"es"` also exist).
- **Icons:** `Botloft.icons.<Name>` holds the lucide icons the app uses (`Plus`, `Pause`, `Settings`,
  `Ellipsis`, `X`, `Archive`, `Trash2`, `RotateCw`, `FileText`, `Folder`, `Users`, `Crown`...). Pass one
  as `icon` to `Button` and `Menu`, or render it: `<icons.FileText size={14} />`.

## Styling: Tailwind utilities on the design tokens

Style your own layout with Tailwind classes. **Only classes compiled into `_ds_bundle.css` exist**:
the app's own plus a fixed layout set. Arbitrary values like `h-[360px]` or `w-[37rem]` do NOT exist;
use `style={{ height: 360 }}` for one-off sizes.

| Family | Classes |
|---|---|
| Surfaces | `bg-canvas` (page), `bg-panel` (cards, dialogs), `bg-sunken` (wells, hover) |
| Text | `text-ink`, `text-ink-soft` (body), `text-muted` (hints), `text-accent` |
| Lines | `border border-line`, `border-line-strong` (controls), `divide-y divide-line` |
| State | `text-ok` ready, `text-work` working, `text-warn` waiting, `text-danger` error, `text-quiet` off; tints like `bg-accent/10`, `bg-danger/10` |
| Type | `text-xs` 12px, `text-sm` 13px, `text-base` 14px (body), `text-lg`...`text-2xl` for titles; `font-medium`, `font-semibold`; `font-mono` for paths and logs |
| Shape | `rounded-lg` (controls), `rounded-xl` (cards, callouts), `rounded-2xl` (dialogs), `rounded-full` (pills); `shadow-lift` for floating surfaces |
| Layout | `flex`, `grid grid-cols-1`...`grid-cols-12`, `gap-*`, `p-*`, `px-*`, `py-*`, `m*-*` on the 0.5-24 scale, `w-*`/`h-*`/`size-*` 0-96, `max-w-xs`...`max-w-7xl` |

The tokens behind them are CSS variables: `--canvas`, `--panel`, `--sunken`, `--line`,
`--line-strong`, `--ink`, `--ink-soft`, `--muted`, `--accent`, `--ok`, `--work`, `--warn`,
`--danger`. Use `var(--accent)` in inline styles; never hard-code hex for UI colors.
Bot colors are the exception: each bot has its own (`#FF7A59` Botloft orange, `#4FC382`, `#6EA6FF`,
`#F0B24A`, `#C084FC`), passed to `BotAvatar`.

## Components and their roles

- `BotAvatar` is the mascot: `color`, `size`, `mood` (idle, working, waiting, tired, sleeping;
  it animates), `framed` for Botloft itself on its black square.
- `BotStateBadge` shows a bot's state with icon and word; `ChiefBadge` marks a crew's lead bot.
- `Button` variants: `primary` (one per view), `secondary`, `ghost` (toolbars), `danger`.
- `Dialog`, `Confirm` are modal (fixed, covering the window). `Callout` for notices in a page,
  `Toaster` + `notifyError(what, error)` for failures away from a form.
- Fields: `TextField`, `TextArea`, `SelectField` (labeled), `Select` (custom dropdown),
  `Choices` (2-6 options side by side), `Switch`, `Tabs`, `Menu`, `Details`, `SidePanel`.

Read `components/<group>/<Name>/<Name>.prompt.md` for each one's props.

## Voice

Plain words for people who are not programmers. Name bots by their name ("Scout is waiting for
you"), never "the agent" or "Claude". Never say "daemon"; say "Botloft" or "in the background".

## Example

```jsx
const { BotAvatar, BotStateBadge, Button, Callout, icons, setLocaleChoice } = window.Botloft;
setLocaleChoice("en");

function CrewCard() {
  return (
    <div className="flex max-w-md flex-col gap-3 rounded-xl border border-line bg-panel p-4">
      <div className="flex items-center gap-3">
        <BotAvatar color="#4FC382" size={36} mood="working" />
        <div className="min-w-0 flex-1">
          <p className="font-semibold text-ink text-sm">Scout</p>
          <BotStateBadge bot={{ state: "busy", paused: false }} compact />
        </div>
        <Button variant="ghost" icon={icons.Pause} label="Pause Scout" />
      </div>
      <Callout tone="warn" title="Scout needs your approval">
        It wants to open arxiv.org for the first time.
      </Callout>
    </div>
  );
}
```
