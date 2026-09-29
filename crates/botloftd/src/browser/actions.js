// What a bot does on the page (spec 21.4), beside the page reader: where to
// click an element, getting a text field ready for typing, choosing an
// option and scrolling to an end. `reader.js` hands in its helpers.
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

  // Where to click an element, after scrolling it into the middle of the
  // screen, and what else is on top of that point, if anything.
  const point = (ref) => {
    const el = get(ref);
    if (!el) return { missing: true, next: next() };
    el.scrollIntoView({ block: "center", inline: "center", behavior: "instant" });
    const box = topRect(el);
    const role = roleOf(el, styleOf(el), "auto") ?? el.localName;
    const label = clean(nameOf(el), 60);
    if (box.width === 0 || box.height === 0) return { hidden: true, role, label, next: next() };
    const x = Math.min(Math.max(box.left + box.width / 2, 0), innerWidth - 1);
    const y = Math.min(Math.max(box.top + box.height / 2, 0), innerHeight - 1);
    const local = el.getBoundingClientRect();
    const hit = el.ownerDocument.elementFromPoint(
      local.left + local.width / 2,
      local.top + local.height / 2,
    );
    let cover = null;
    if (hit && hit !== el && !el.contains(hit) && !hit.contains(el)) {
      let shown = hit;
      while (shown && !roleOf(shown, styleOf(shown), "auto")) shown = shown.parentElement;
      cover = shown ? token(shown, roleOf(shown, styleOf(shown), "auto")) : `<${hit.localName}>`;
    }
    return { x, y, role, label, text: isTextField(el), cover, next: next() };
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

  return { point, prepareType, select, scrollEnd };
}
