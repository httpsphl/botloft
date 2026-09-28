// The bot's terminal: xterm.js on top of a TerminalSession (spec 8). The
// screen stays dark in both themes; Claude Code picks its colors for a
// dark terminal.

import "@xterm/xterm/css/xterm.css";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import { type ITheme, Terminal } from "@xterm/xterm";
import { useEffect, useRef } from "react";
import type { BotId } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { TerminalSession } from "./session";

const THEME: ITheme = {
  background: "#0b0b0b",
  foreground: "#e6e6e1",
  cursor: "#ff7a59",
  cursorAccent: "#0b0b0b",
  selectionBackground: "#ff7a5955",
};

/** Ctrl+C copies when text is selected, as in Windows Terminal. */
function copyOnCtrlC(terminal: Terminal) {
  return (event: KeyboardEvent): boolean => {
    const copy = event.type === "keydown" && event.ctrlKey && !event.shiftKey && event.key === "c";
    if (copy && terminal.hasSelection()) {
      void navigator.clipboard.writeText(terminal.getSelection());
      terminal.clearSelection();
      return false;
    }
    return true;
  };
}

export function TerminalView({ botId }: { botId: BotId }) {
  const api = useApi();
  const container = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }
    const terminal = new Terminal({
      fontFamily: '"Cascadia Mono", Consolas, "IBM Plex Mono", monospace',
      fontSize: 13,
      lineHeight: 1.1,
      cursorBlink: true,
      scrollback: 5000,
      theme: THEME,
      windowsPty: { backend: "conpty" },
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(element);
    try {
      const webgl = new WebglAddon();
      webgl.onContextLoss(() => webgl.dispose());
      terminal.loadAddon(webgl);
    } catch {
      // Without WebGL xterm draws with the DOM renderer.
    }

    // While xterm parses replayed output, what it "types" are answers to
    // queries asked long ago (cursor position, device attributes...). The
    // bot would read them as keys, so they are dropped.
    let replaying = 0;
    const session = new TerminalSession(api, botId, {
      write: (data, history) => {
        if (history) {
          replaying += 1;
          terminal.write(data, () => {
            replaying -= 1;
          });
        } else {
          terminal.write(data);
        }
      },
      reset: () => terminal.reset(),
    });
    const subscriptions = [
      terminal.onData((data) => replaying === 0 && session.input(data)),
      terminal.onBinary((data) => replaying === 0 && session.inputBinary(data)),
      terminal.onResize(({ cols, rows }) => session.resize(cols, rows)),
    ];
    terminal.attachCustomKeyEventHandler(copyOnCtrlC(terminal));

    const refit = () => {
      // A hidden or zero-sized container has no dimensions to fit.
      if (element.clientWidth > 0 && element.clientHeight > 0) {
        fit.fit();
      }
    };
    const observer = new ResizeObserver(refit);
    observer.observe(element);
    refit();
    session.resize(terminal.cols, terminal.rows);
    const stop = session.start();
    terminal.focus();

    return () => {
      stop();
      observer.disconnect();
      for (const subscription of subscriptions) {
        subscription.dispose();
      }
      terminal.dispose();
    };
  }, [api, botId]);

  return (
    <div className="min-h-0 flex-1 bg-[#0b0b0b] py-2 pl-3">
      <div ref={container} className="h-full w-full" data-selectable />
    </div>
  );
}
