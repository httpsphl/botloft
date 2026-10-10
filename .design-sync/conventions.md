# Building with Botloft

Botloft is a desktop app where a crew of AI agents works together. Each agent is a small flame
mascot in its own color. The look is quiet and warm: neutral surfaces, one orange accent, IBM Plex.

## Setup

- Link `styles.css` and load `_ds_bundle.js`; components are on `window.Botloft`. The basic pieces
  (Button, fields, Dialog, Callout, BotAvatar, BotStateBadge...) need nothing around them.
- **App screens need `BotloftProvider`.** `Sidebar`, `CrewBots`, `BotRun`, `InboundRow` and
  `ApprovalCard` are the app's real screens and read the app's data: wrap them in one
  `<BotloftProvider crews={[...]} agents={[...]}>` per design (not one per piece). Build the data
  with `makeCrew`, `makeBot`, `makeMessage` and `chat.*` (below); never write an agent object by hand.
  A new `agents` array updates every screen, which is how a design animates states.
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
Agent colors are the exception: each agent has its own from `Botloft.AVATAR_PALETTE` (`#FF7A59`
Botloft orange, `#5EC8FF`, `#A48BFF`, `#9BE564`, `#FFC857`, `#FF7EB6`, `#3DD9C1`, `#E8D5B0`);
`makeBot` hands them out in that order.

## Data for the app screens

```jsx
const { makeCrew, makeBot, makeMessage, chat } = window.Botloft;
const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true, state: "idle" });
const scout = makeBot({ crew, name: "Scout", state: "busy", role: "Finds sources.",
  activity: { kind: "tool", text: "on-device AI news", tool: "WebSearch" } }); // sidebar line
```

- `state`: `idle`, `busy` (working), `needs_approval`, `launching`, `rate_limited`, `backoff`,
  `auth_error`, `offline` (with `paused: true` it reads Paused).
- An agent's turn is a list for `BotRun`: `chat.reply(agent, markdown)`, `chat.tool(agent, { name:
  "WebSearch" | "WebFetch" | "Read" | "Write" | "Bash", summary })`, `chat.askSite(agent, url)`,
  `chat.askCommand(agent, command, { explanation })`, `chat.suggestBot(chief, { name, role, model,
  instructions, reason })`, `chat.askPlan(agent, markdown)`, `chat.turn(agent, seconds)`. Each takes
  `{ status: "allowed" | "denied" }` to show it answered. `ApprovalCard` takes one of them's `.body`.
  `chat.askRoutine(agent, { name, prompt, schedule, agent: "writer" })` is an agent asking to set up a
  routine (for itself, or for another agent by handle).
- Routines, for `CrewRoutines` (pass them as `BotloftProvider` `routines`): `makeRoutine({ agent,
  name, prompt, schedule: schedule.weekly([5], "09:00") | schedule.weekdays("08:30") |
  schedule.every(120), lastRun: { status: "done" | "failed" | "queued" | "skipped" } })`. Days are
  1 = Monday ... 7 = Sunday. The next run follows the schedule. `status: "queued"` reads "Running
  now": a new `routines` array with it shows a routine starting.
- `makeMessage({ from: agent | "owner" | "system", to: agent, body, kind: "note" | "task" })` for
  `InboundRow`: yours on the right, an agent's or Botloft's on the left.

## Components and their roles

- `BotAvatar` is the mascot: `color`, `size`, `mood` (idle, working, waiting, tired, sleeping;
  it animates), `framed` for Botloft itself on its black square.
- `BotStateBadge` shows an agent's state with icon and word; `ChiefBadge` marks a crew's lead agent.
- `Button` variants: `primary` (one per view), `secondary`, `ghost` (toolbars), `danger`.
- `Dialog`, `Confirm` are modal (fixed, covering the window). `Callout` for notices in a page,
  `Toaster` + `notifyError(what, error)` for failures away from a form.
- Fields: `TextField`, `TextArea`, `SelectField` (labeled), `Select` (custom dropdown),
  `Choices` (2-6 options side by side), `Switch`, `Tabs`, `Menu`, `Details`, `SidePanel`.
- App screens: `Sidebar` (crews and agents, 288px wide), `CrewBots` (a crew's agent cards), `BotRun`
  (one turn of an agent in its chat, inside a `<ul>`), `InboundRow` (a message in a chat, inside a
  `<ul>`), `ApprovalCard` (a request: a website, a command, a suggested agent, a plan, a routine),
  `BrowserPanel` (an agent's browser, live: tabs, address bar, the page and the agent's cursor),
  `CrewRoutines` (a crew's routines: each with its agent, schedule in words, next run, last run;
  it fills the chat column's place, `<CrewRoutines crew={crew} />`).
  `BotloftProvider`'s `onAnswer` hears Allow and Deny on request cards.
- **The app window:** one row, `className="flex bg-canvas text-ink"` with a fixed height
  (`style={{ height: 820 }}`): the `Sidebar` as its **direct first child**, then the chat column
  (`flex min-w-0 flex-1 flex-col`), then, when an agent's browser is open, `<BrowserPanel agent={...}
  onClose={...} />` as the **last direct child**. The Sidebar stretches to the row's full height,
  with the account pinned at the bottom-left; wrapped in any other element it shrinks to its
  content and looks cut short. Never wrap the Sidebar or the BrowserPanel. Give the window at
  least 1400px of width when the browser is open, so the chat keeps its room.
- **A browser:** pass `browsers={[{ agent: scout, url, title, tabs: [{ url, title }], action: { kind:
  "click", x, y, label } }]}` to `BotloftProvider`. The page is drawn for you (title, address,
  text lines); `action` moves the agent's cursor (`open`, `click`, `type`, `scroll`, `back`), and a
  new `browsers` array with another `url` or `action` makes the agent browse on.

Read `components/<group>/<Name>/<Name>.prompt.md` for each one's props.

## Voice

Plain words for people who are not programmers. Name agents by their name ("Scout is waiting for
you"), never "the agent" or "Claude". Never say "daemon"; say "Botloft" or "in the background".

## Example

```jsx
const { BotloftProvider, Sidebar, BotRun, BotStateBadge, makeCrew, makeBot, chat, setLocaleChoice } =
  window.Botloft;
setLocaleChoice("en");
const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true });
const scout = makeBot({ crew, name: "Scout", state: "needs_approval" });
const site = chat.askSite(scout, "https://arxiv.org/abs/2609.01234");

function AppWindow() {
  const [allowed, setAllowed] = React.useState(false);
  const scoutNow = allowed ? { ...scout, state: "busy" } : scout;
  const card = allowed ? { ...site, body: { ...site.body, status: "allowed" } } : site;
  return (
    <BotloftProvider crews={[crew]} agents={[chief, scoutNow]} selectedBotId={scout.id}
      onAnswer={(answer) => setAllowed(answer.allow)}>
      <div className="flex bg-canvas text-ink" style={{ height: 820 }}>
        <Sidebar /> {/* direct child of the row: it runs the full height */}
        <ul className="flex min-w-0 flex-1 flex-col gap-5 p-5">
          <BotRun agent={scoutNow} items={[chat.reply(scout, "I need to open this paper."), card]}
            live={{ draft: "", working: allowed }} />
        </ul>
      </div>
    </BotloftProvider>
  );
}
```
