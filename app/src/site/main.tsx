// The site's script (spec 39.4). The page is already whole without it:
// this offers the visitor's own download, remembers a chosen language,
// wakes the mascots and plays the demos, each at its end for a visitor who
// asks for less motion.

import { createRoot, type Root } from "react-dom/client";
import { BotAvatar, type Mood } from "../features/bots/BotAvatar";
import { LOCALE_KEY, localeOfPath } from "./locales";
import { type MascotProps, readMascot } from "./Mascot";
import { playScenes } from "./player";
import { MAIN_DOWNLOAD, systemOf } from "./systems";
import "./site.css";

// While the site is worked on (`pnpm dev:site`) there is no built page:
// draw it here, as the build would.
if (import.meta.env.DEV) {
  const root = document.getElementById("site");
  if (root && !root.firstElementChild) {
    const [{ renderToStaticMarkup }, { Page }] = await Promise.all([
      import("react-dom/server"),
      import("./Page"),
    ]);
    const locale = localeOfPath(window.location.pathname);
    document.documentElement.lang = locale;
    root.innerHTML = renderToStaticMarkup(<Page locale={locale} />);
  }
}

interface PlatformHint {
  userAgentData?: { platform?: string };
}

/** The visitor's system gets its own download first. */
function offerDownload(): void {
  const hint = (navigator as Navigator & PlatformHint).userAgentData?.platform;
  const system = systemOf(
    navigator.userAgent,
    hint ?? navigator.platform,
    navigator.maxTouchPoints,
  );
  for (const button of document.querySelectorAll<HTMLElement>("[data-system]")) {
    button.hidden = button.dataset.system !== system;
  }
  for (const link of document.querySelectorAll<HTMLAnchorElement>("[data-download]")) {
    link.href = MAIN_DOWNLOAD[system];
  }
}

/** A language picked on the switch stays picked, so the English page sends no one away. */
function rememberLanguage(): void {
  for (const link of document.querySelectorAll<HTMLAnchorElement>("[data-locale]")) {
    link.addEventListener("click", () => {
      try {
        localStorage.setItem(LOCALE_KEY, link.dataset.locale ?? "en");
      } catch {
        // Without storage the visitor's own language still decides.
      }
    });
  }
}

function draw(root: Root, mascot: MascotProps): void {
  root.render(<BotAvatar color={mascot.color} size={mascot.size} mood={mascot.mood} />);
}

/**
 * Each drawn mascot becomes a live one, with the same color, size and mood;
 * a demo changes its mood on its steps (player.ts).
 */
function wakeMascots(): void {
  for (const holder of document.querySelectorAll<HTMLElement>(".site-mascot")) {
    const mascot = readMascot(holder);
    if (!mascot) continue;
    holder.replaceChildren();
    const root = createRoot(holder);
    draw(root, mascot);
    holder.addEventListener("site:mood", (event) => {
      const mood = (event as CustomEvent<Mood>).detail;
      draw(root, { ...mascot, mood });
    });
  }
}

const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
document.documentElement.classList.add("site-js");
offerDownload();
rememberLanguage();
wakeMascots();
playScenes(document, still);
