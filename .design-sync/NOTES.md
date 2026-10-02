# Design sync notes

Botloft is an app, not a component library, so the design system is a small package built from the
app's own source: `.design-sync/pkg/` (`index.ts` re-exports the components, `styles.css` adds a
layout vocabulary to the app's stylesheet, `build.mjs` builds `dist/` with the app's Vite,
Tailwind and TypeScript). Nothing there reimplements a component.

## Running it

- `node .design-sync/pkg/build.mjs` first (cfg.buildCmd), then the converter with
  `--node-modules ./app/node_modules --entry ./.design-sync/pkg/dist/index.js`.
- `app/node_modules` must be installed (`pnpm install` in `app/`).
- `.design-sync/node_modules` is a junction to `.ds-sync/node_modules` (gitignored, recreate per
  clone). Without it `@types/react` is not found from `.design-sync/pkg` and React props collapse
  ([DTS_REACT]). On Windows: `node -e "require('fs').symlinkSync(require('path').resolve('.ds-sync/node_modules'), '.design-sync/node_modules', 'junction')"`.
- Render check and capture use `playwright@1.60.0` in `.ds-sync/` (matches the cached chromium-1223).

## Fixes made in this repo's setup

- Vite library mode inlines every asset, so the fonts would land in the CSS as base64 (1.1 MB, every
  alphabet). `build.mjs` aliases `@fontsource/*.css` to an empty file and the Latin subsets ship as
  files through cfg.extraFonts.
- `icons.gen.ts` is generated from the app's `lucide-react` imports and exported as `icons`
  (a namespace, so the 94 icons do not become component cards). `lucide-react` is aliased to
  `app/node_modules` because the file sits outside the app.
- `.d.ts` emit: tsc from `app/node_modules/typescript` (TypeScript 7; `bin/tsc` is not in its
  exports map, so the path is built from `typescript/package.json`).
- Props that extend HTML attributes (`Button`, `TextField`, `TextArea`, `SelectField`) lose
  `onClick`/`value`/`onChange` in extraction, and `BotStateBadge`, `ChiefBadge`, `Menu` reference
  app types (`Bot`, `Crew`, `LucideIcon`). All six have hand-written bodies in cfg.dtsPropsFor.
  Keep them in step with the app's source.
- Only Tailwind classes compiled into the CSS exist. The app's own set missed common layout
  utilities (`max-w-sm`, arbitrary sizes), so `pkg/styles.css` adds a fixed vocabulary with
  `@source inline(...)` and scans `.design-sync/previews`.
- Components speak the browser's language. Previews call `setLocaleChoice("en")` (exported from
  the package) so cards are English on any machine.
- Fixed overlays (`Dialog`, `Confirm`, `Toaster`) render inside a box with `transform:
  translateZ(0)`, which makes it their containing block; without it the per-story capture is
  0 px tall.
- `ListAvatar` is excluded (cfg.componentSrcMap): a wrapper over `BotAvatar` that needs a bot object.

## Known render warns

- `[FONT_MISSING] "Cascadia Mono"`: a system fallback after IBM Plex Mono in `--font-mono`; Plex
  Mono ships. Accepted.
- `Menu` shows only its closed button: opening takes a click.
- `SelectField` Disabled looks like the enabled one: the app's own control does not style disabled.

## Re-sync risks

- `dtsPropsFor` bodies are copies of the app's props; a prop added in the app does not reach the
  contract until the body is updated.
- The layout vocabulary in `pkg/styles.css` is a fixed list; the conventions header names its
  ranges. Change both together.
- The icon set follows the app's imports: an icon the app stops using disappears from `icons`.
- Bot colors in previews and conventions are sample values, not tokens.
- The scope is the base pieces plus the bot badges (owner's choice on 2026-10-02). Chat rows,
  approval cards and the sidebar are not in the package.
