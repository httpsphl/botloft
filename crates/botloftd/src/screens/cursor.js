// The bot's cursor on a screen it is writing (spec 22.3). The daemon puts
// this first in every draft it serves. Once the page is read, it finds where
// the content ends right now (the end of the newest text, or the newest
// element), keeps that spot in view and tells the app where it is and which
// part of the page holds it, in the page's pixels, with the size of the view
// they are measured in. It changes nothing else.
(() => {
  const SKIP = new Set(["SCRIPT", "STYLE", "TEMPLATE", "NOSCRIPT", "BR", "WBR"]);
  const shown = (rect) => rect !== undefined && (rect.width > 0 || rect.height > 0);

  const end = () => {
    const body = document.body;
    if (!body) {
      return null;
    }
    const walk = document.createTreeWalker(body, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
    const nodes = [];
    for (let node = walk.nextNode(); node; node = walk.nextNode()) {
      nodes.push(node);
    }
    for (let at = nodes.length - 1; at >= 0; at -= 1) {
      const node = nodes[at];
      const element = node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
      if (!element || element.closest("script, style, template, noscript") || SKIP.has(element.tagName)) {
        continue;
      }
      if (node.nodeType === Node.TEXT_NODE) {
        if (!node.data.trim()) {
          continue;
        }
        const range = document.createRange();
        range.selectNodeContents(node);
        const lines = range.getClientRects();
        const last = lines[lines.length - 1];
        if (shown(last)) {
          return { element, x: last.right, y: last.top + last.height / 2 };
        }
      } else {
        const box = element.getBoundingClientRect();
        if (shown(box)) {
          return { element, x: box.left + box.width / 2, y: box.top + box.height / 2 };
        }
      }
    }
    return null;
  };

  // The part of the page around the spot: the nearest element that is not
  // inline text, short of the whole page.
  const part = (element) => {
    let current = element;
    while (current.parentElement && getComputedStyle(current).display === "inline") {
      current = current.parentElement;
    }
    return current === document.body || current === document.documentElement ? null : current;
  };

  const follow = () => {
    // A frame from another origin runs in its own process and learns its
    // size a moment after the page is read: measure once it has one.
    if (innerWidth === 0 || innerHeight === 0) {
      addEventListener("resize", follow, { once: true });
      return;
    }
    let spot = end();
    if (spot && (spot.y > innerHeight * 0.85 || spot.y < 0)) {
      scrollTo({ top: scrollY + spot.y - innerHeight * 0.6, behavior: "instant" });
      spot = end();
    }
    const around = spot && part(spot.element);
    const box = around?.getBoundingClientRect();
    parent.postMessage(
      {
        botloft: "cursor",
        view: { width: innerWidth, height: innerHeight },
        point: spot ? { x: spot.x, y: spot.y } : null,
        box: box ? { x: box.left, y: box.top, width: box.width, height: box.height } : null,
      },
      "*",
    );
  };

  addEventListener("DOMContentLoaded", follow);
  // Images and fonts move things once they arrive.
  addEventListener("load", follow);
})();
