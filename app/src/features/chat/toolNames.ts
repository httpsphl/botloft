// A tool in the owner's language (spec 15.6): its name as an action ("Send
// a message", not `send_message`) and what it is about. The daemon's
// summaries carry no wording; where one would need words, they are made
// here from the tool's input.

import type { Messages } from "../../i18n/en";

type Words = Messages["tools"];

/** A tool of the owner's, `mcp__<slug>__<tool>`, as "tool name (slug)". */
function connected(name: string): string | null {
  const [prefix, slug, ...tool] = name.split("__");
  if (prefix !== "mcp" || !slug || slug === "botloft" || tool.length === 0) {
    return null;
  }
  return `${tool.join("__").replaceAll("_", " ")} (${slug})`;
}

/** `mcp__botloft__send_message` is `send_message`; built-ins keep theirs. */
export function toolKey(name: string): string {
  return name.split("__").at(-1) ?? name;
}

function known(key: string, t: Words): string | undefined {
  return Object.hasOwn(t.names, key) ? t.names[key as keyof Words["names"]] : undefined;
}

/** The tool as a title: "Send a message"; an unknown one, by its name. */
export function toolTitle(name: string, t: Words): string {
  const key = toolKey(name);
  return known(key, t) ?? connected(name) ?? key;
}

/** The tool inside a sentence: "wants to send a message". */
export function toolAction(name: string, t: Words): string {
  const key = toolKey(name);
  const title = known(key, t);
  return title ? title.charAt(0).toLowerCase() + title.slice(1) : t.use(key);
}

function input(tool: { input: string }): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(tool.input);
    return typeof parsed === "object" && parsed !== null ? (parsed as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

/** What the call is about, after the title. */
export function toolDetail(
  tool: { name: string; summary: string; input: string },
  t: Words,
): string {
  switch (toolKey(tool.name)) {
    case "send_message": {
      const to = input(tool).to;
      return typeof to === "string" ? `@${to.replace(/^@/, "")}` : "";
    }
    // Refs like "e12" mean nothing to the owner: the browser panel shows
    // what the bot clicked, by name.
    case "complete_task":
    case "TodoWrite":
    case "browser_click":
    case "browser_type":
    case "browser_select":
      return "";
    case "browser_scroll": {
      const to = input(tool).to;
      return typeof to === "string" && Object.hasOwn(t.scroll, to)
        ? t.scroll[to as keyof Words["scroll"]]
        : "";
    }
    case "ToolSearch": {
      const query = input(tool).query;
      if (typeof query === "string" && query.startsWith("select:")) {
        return query
          .slice("select:".length)
          .split(",")
          .map((name) => toolTitle(name.trim(), t))
          .join(", ");
      }
      return tool.summary;
    }
    default:
      return tool.summary;
  }
}
