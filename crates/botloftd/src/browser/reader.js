// Botloft's page reader (spec 21.6). It runs in an isolated world of the
// bot's browser, so the page's own scripts neither see it nor change the
// built-ins it uses. `(install)(start, actions)` sets it up once per
// document, with what `actions.js` adds; refs count up from `start`, which
// the daemon keeps per browser, so a ref never names two elements.
(start, actions) => {
  if (window.__botloft) {
    return window.__botloft;
  }
  const refs = new Map();
  const ids = new WeakMap();
  let next = start;

  const SKIP = new Set([
    "script", "style", "noscript", "template", "head", "meta", "link", "title", "base",
    "param", "source", "track", "datalist", "option", "optgroup",
  ]);
  const ROLES = new Set([
    "button", "link", "checkbox", "radio", "tab", "menuitem", "menuitemcheckbox",
    "menuitemradio", "option", "switch", "combobox", "textbox", "searchbox", "slider",
    "spinbutton", "treeitem",
  ]);
  const TEXT_INPUTS = new Set([
    "text", "search", "email", "url", "tel", "password", "number", "date", "datetime-local",
    "month", "week", "time",
  ]);
  const MAX_NODES = 40000;
  const MAX_DEPTH = 300;

  const clean = (text, max) => {
    const flat = String(text ?? "").replace(/\s+/g, " ").trim().replace(/"/g, "'");
    return flat.length > max ? `${flat.slice(0, max - 1)}…` : flat;
  };
  const refOf = (el) => {
    let ref = ids.get(el);
    if (!ref) {
      ref = `e${next++}`;
      ids.set(el, ref);
      refs.set(ref, new WeakRef(el));
    }
    return ref;
  };
  const get = (ref) => {
    const el = refs.get(String(ref).trim())?.deref();
    return el?.isConnected ? el : null;
  };
  const styleOf = (el) => el.ownerDocument.defaultView.getComputedStyle(el);

  const inputType = (el) => (el.getAttribute("type") || "text").toLowerCase();
  const isTextField = (el) =>
    el.localName === "textarea" ||
    (el.localName === "input" && TEXT_INPUTS.has(inputType(el))) ||
    (el.isContentEditable && !el.parentElement?.isContentEditable);

  const roleOf = (el, style, parentCursor) => {
    const aria = (el.getAttribute("role") || "").trim().split(/\s+/)[0];
    if (ROLES.has(aria)) {
      return aria === "searchbox" ? "textbox" : aria;
    }
    const tag = el.localName;
    if (tag === "a" && el.hasAttribute("href")) return "link";
    if (tag === "button" || tag === "summary") return "button";
    if (tag === "select") return "select";
    if (tag === "textarea") return "textbox";
    if (tag === "input") {
      const type = inputType(el);
      if (type === "hidden") return null;
      if (["button", "submit", "reset", "image"].includes(type)) return "button";
      if (type === "checkbox" || type === "radio") return type;
      if (type === "range") return "slider";
      if (type === "file") return "file";
      return TEXT_INPUTS.has(type) ? "textbox" : "input";
    }
    if (el.isContentEditable && !el.parentElement?.isContentEditable) return "textbox";
    if (el.hasAttribute("onclick")) return "clickable";
    if (style.cursor === "pointer" && parentCursor !== "pointer") return "clickable";
    return null;
  };

  const nameOf = (el) => {
    const aria = el.getAttribute("aria-label");
    if (aria?.trim()) return aria;
    const by = el.getAttribute("aria-labelledby");
    if (by) {
      const text = by
        .split(/\s+/)
        .map((id) => el.ownerDocument.getElementById(id)?.innerText ?? "")
        .join(" ")
        .trim();
      if (text) return text;
    }
    if (el.labels?.length) {
      // A label around the field holds the field too: leave its text out.
      const text = [...el.labels]
        .map((label) => {
          const copy = label.cloneNode(true);
          for (const inner of copy.querySelectorAll("input, select, textarea, button")) {
            inner.remove();
          }
          return copy.textContent;
        })
        .join(" ")
        .trim();
      if (text) return text;
    }
    const tag = el.localName;
    if (tag === "input" && ["button", "submit", "reset"].includes(inputType(el))) {
      return el.value || inputType(el);
    }
    if (tag !== "input" && tag !== "textarea" && tag !== "select") {
      const text = el.innerText?.trim();
      if (text) return text;
    }
    return (
      el.getAttribute("placeholder") ||
      el.getAttribute("title") ||
      el.getAttribute("alt") ||
      el.querySelector?.("img[alt]")?.getAttribute("alt") ||
      el.getAttribute("name") ||
      ""
    );
  };

  const token = (el, role) => {
    let out = `[${refOf(el)} ${role}`;
    const name = clean(nameOf(el), 80);
    if (name) out += ` "${name}"`;
    if (role === "textbox") {
      if (el.localName === "input" && inputType(el) === "password") {
        out += el.value ? " (password, filled)" : " (password)";
      } else {
        const value = el.isContentEditable ? el.innerText : el.value;
        out += ` = "${clean(value, 60)}"`;
      }
    }
    if (role === "select") {
      const chosen = el.selectedOptions?.[0]?.text;
      out += ` = "${clean(chosen, 60)}"`;
      const options = [...el.options].slice(0, 8).map((option) => clean(option.text, 30));
      if (options.length) out += ` options: ${options.join(" | ")}${el.options.length > 8 ? " | …" : ""}`;
    }
    const checked = el.checked ?? el.getAttribute("aria-checked") === "true";
    if (checked && ["checkbox", "radio", "switch", "menuitemcheckbox", "menuitemradio"].includes(role)) {
      out += " (checked)";
    }
    if (el.disabled || el.getAttribute("aria-disabled") === "true") out += " (disabled)";
    const expanded = el.getAttribute("aria-expanded");
    if (expanded === "true") out += " (expanded)";
    if (expanded === "false") out += " (collapsed)";
    return `${out}]`;
  };

  const read = (from, max) => {
    const out = [];
    let nodes = 0;
    const block = (display) => !display.startsWith("inline") && display !== "contents";

    const walkChildren = (parent, cursor, depth) => {
      const root = parent.shadowRoot ?? parent;
      for (const child of root.childNodes) {
        walk(child, cursor, depth + 1);
      }
    };

    const walk = (node, parentCursor, depth) => {
      if (++nodes > MAX_NODES || depth > MAX_DEPTH) return;
      if (node.nodeType === Node.TEXT_NODE) {
        const text = node.nodeValue.replace(/\s+/g, " ");
        if (text.trim()) out.push(text);
        return;
      }
      if (node.nodeType !== Node.ELEMENT_NODE) return;
      const el = node;
      const tag = el.localName;
      if (SKIP.has(tag)) return;
      if (tag === "br") {
        out.push("\n");
        return;
      }
      if (el.getAttribute("aria-hidden") === "true") return;
      const style = styleOf(el);
      if (style.display === "none" || style.visibility === "hidden" || style.visibility === "collapse") {
        return;
      }
      if (tag === "slot") {
        const assigned = el.assignedNodes({ flatten: true });
        for (const child of assigned.length ? assigned : el.childNodes) {
          walk(child, parentCursor, depth + 1);
        }
        return;
      }
      const rect = el.getBoundingClientRect();
      const empty = rect.width === 0 && rect.height === 0 && style.display !== "contents";
      if (empty && style.overflow !== "visible") return;
      const role = roleOf(el, style, parentCursor);
      if (role && !empty) {
        const long = role === "clickable" && (el.innerText?.length ?? 0) > 80;
        out.push(` ${long ? `[${refOf(el)} clickable]` : token(el, role)} `);
        if (!long) return;
      }
      if (tag === "img" || tag === "svg") {
        const alt = clean(el.getAttribute("alt") || el.getAttribute("aria-label") || el.querySelector?.("title")?.textContent, 80);
        if (alt) out.push(` [image "${alt}"] `);
        return;
      }
      if (tag === "video" || tag === "canvas") {
        out.push(` [${tag}] `);
        return;
      }
      if (tag === "iframe") {
        let inner = null;
        try {
          inner = el.contentDocument?.body ?? null;
        } catch {
          inner = null;
        }
        if (inner) {
          out.push("\n");
          walk(inner, "auto", depth + 1);
          out.push("\n");
        } else {
          out.push(" [frame] ");
        }
        return;
      }
      const isBlock = block(style.display);
      const heading = /^h([1-6])$/.exec(tag);
      if (isBlock) out.push("\n");
      if (heading) out.push(`${"#".repeat(Number(heading[1]))} `);
      if (tag === "li") out.push("- ");
      walkChildren(el, style.cursor, depth);
      if (tag === "td" || tag === "th") out.push(" | ");
      if (isBlock) out.push("\n");
    };

    const body = document.body ?? document.documentElement;
    if (body) walk(body, "auto", 0);
    const text = out
      .join("")
      .replace(/[ \t ]+/g, " ")
      .replace(/ *\n */g, "\n")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
    return {
      title: document.title,
      url: location.href,
      text: text.slice(from, from + max),
      total: text.length,
      next,
    };
  };

  const api = {
    read,
    ...actions({ get, clean, styleOf, roleOf, nameOf, token, isTextField, next: () => next }),
  };
  Object.defineProperty(window, "__botloft", { value: api });
  return api;
}
