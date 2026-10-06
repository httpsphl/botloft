// What a bot does on the page (spec 21.4), beside the page reader: where to
// click an element, getting a text field ready for typing, choosing an
// option and scrolling to an end; and, for a lesson (spec 21.13), what is
// at a point and which field has the focus. `reader.js` hands in its
// helpers.
({ get, clean, styleOf, roleOf, nameOf, token, isTextField, next }) => {
  // The element's box in the top page's coordinates, through same-site frames.
  const topRect = (el) => {
    const rect = el.getBoundingClientRect();
    let left = rect.left;
    let top = rect.top;
    let view = el.ownerDocument.defaultView;
    while (view && view.frameElement) {
      const frame = view.frameElement.getBoundingClientRect();
      left += frame.left + view.frameElement.clientLeft;
      top += frame.top + view.frameElement.clientTop;
      view = view.parent;
    }
    return { left, top, width: rect.width, height: rect.height };
  };

  // Whether something else is on top of a point of an element: true when
  // the point shows the element, a part of it or what it sits in.
  const shows = (el, x, y) => {
    const hit = el.ownerDocument.elementFromPoint(x, y);
    return !!hit && (hit === el || el.contains(hit) || hit.contains(el));
  };

  // A point inside the element that is not covered, the middle first, then
  // the corners and edges of its middle part, or null. Returns the fraction
  // of the box it is at.
  const freePoint = (el, local) => {
    const view = el.ownerDocument.defaultView;
    for (const fy of [0.5, 0.2, 0.8]) {
      for (const fx of [0.5, 0.2, 0.8]) {
        const x = local.left + local.width * fx;
        const y = local.top + local.height * fy;
        if (x < 0 || y < 0 || x >= view.innerWidth || y >= view.innerHeight) continue;
        if (shows(el, x, y)) return { fx, fy };
      }
    }
    return null;
  };

  // Where to click an element. It scrolls into the middle of the screen;
  // when a fixed header, a banner or a chat widget covers it there, it tries
  // the other alignments and the other points of the element, and when
  // nothing is free it says so (`script`), so the click can go through the
  // page instead of the mouse. Also what is still on top, if anything.
  const point = (ref) => {
    const el = get(ref);
    if (!el) return { missing: true, next: next() };
    let pick = null;
    for (const block of ["center", "end", "start", "nearest"]) {
      el.scrollIntoView({ block, inline: "center", behavior: "instant" });
      const local = el.getBoundingClientRect();
      if (local.width === 0 || local.height === 0) break;
      pick = freePoint(el, local);
      if (pick) break;
    }
    if (!pick) el.scrollIntoView({ block: "center", inline: "center", behavior: "instant" });
    const box = topRect(el);
    const role = roleOf(el, styleOf(el), "auto") ?? el.localName;
    const label = clean(nameOf(el), 60);
    if (box.width === 0 || box.height === 0) return { hidden: true, role, label, next: next() };
    const [fx, fy] = pick ? [pick.fx, pick.fy] : [0.5, 0.5];
    const x = Math.min(Math.max(box.left + box.width * fx, 0), innerWidth - 1);
    const y = Math.min(Math.max(box.top + box.height * fy, 0), innerHeight - 1);
    const local = el.getBoundingClientRect();
    const hit = el.ownerDocument.elementFromPoint(
      local.left + local.width * fx,
      local.top + local.height * fy,
    );
    let cover = null;
    if (!pick && hit && hit !== el && !el.contains(hit) && !hit.contains(el)) {
      let shown = hit;
      while (shown && !roleOf(shown, styleOf(shown), "auto")) shown = shown.parentElement;
      cover = shown ? token(shown, roleOf(shown, styleOf(shown), "auto")) : `<${hit.localName}>`;
    }
    return { x, y, role, label, text: isTextField(el), cover, script: !pick && cover !== null, next: next() };
  };

  // The page's own click on an element, for when the mouse cannot reach it.
  const clickScript = (ref) => {
    const el = get(ref);
    if (!el) return { missing: true };
    el.click();
    return { ok: true };
  };

  // Focuses a text field and selects what it holds, so typing replaces it.
  const prepareType = (ref) => {
    const el = get(ref);
    if (!el) return { missing: true };
    if (!isTextField(el)) return { notText: true, role: roleOf(el, styleOf(el), "auto") ?? el.localName };
    el.focus();
    const had = el.isContentEditable ? el.innerText : el.value;
    if (el.localName === "input" || el.localName === "textarea") {
      try {
        el.select();
      } catch {
        // Some input types cannot be selected; typing still replaces nothing.
      }
    } else {
      const range = el.ownerDocument.createRange();
      range.selectNodeContents(el);
      const selection = el.ownerDocument.defaultView.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
    }
    return { ok: true, hadText: String(had ?? "").length > 0, label: clean(nameOf(el), 60) };
  };

  const select = (ref, option) => {
    const el = get(ref);
    if (!el) return { missing: true };
    if (el.localName !== "select") return { notSelect: true, role: roleOf(el, styleOf(el), "auto") ?? el.localName };
    const want = String(option).trim().toLowerCase();
    const options = [...el.options];
    const found =
      options.find((o) => o.text.trim().toLowerCase() === want) ??
      options.find((o) => o.value.toLowerCase() === want) ??
      options.find((o) => o.text.trim().toLowerCase().includes(want));
    if (!found) return { noOption: true, options: options.slice(0, 40).map((o) => clean(o.text, 60)) };
    const setter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value").set;
    setter.call(el, found.value);
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return { chosen: clean(found.text, 60), label: clean(nameOf(el), 60) };
  };

  const scrollEnd = (where) => {
    const page = document.scrollingElement ?? document.documentElement;
    window.scrollTo({ top: where === "top" ? 0 : page.scrollHeight, behavior: "instant" });
    return { top: window.scrollY };
  };

  // What the owner acts on while teaching a task (spec 21.13): the nearest
  // element with a role, by its name. Never what a field holds.
  const meaning = (el) => {
    let shown = el;
    while (shown && !roleOf(shown, styleOf(shown), "auto")) shown = shown.parentElement;
    if (!shown) return { none: true };
    return {
      role: roleOf(shown, styleOf(shown), "auto"),
      label: clean(nameOf(shown), 80),
      secret:
        shown.localName === "input" && (shown.getAttribute("type") || "").toLowerCase() === "password",
    };
  };
  const at = (x, y) => meaning(document.elementFromPoint(x, y));
  const focused = () => {
    let el = document.activeElement;
    while (el?.shadowRoot?.activeElement) el = el.shadowRoot.activeElement;
    return el && el !== document.body && isTextField(el) ? meaning(el) : { none: true };
  };

  return { point, clickScript, prepareType, select, scrollEnd, at, focused };
}
