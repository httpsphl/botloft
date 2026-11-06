// The site's pages, drawn when it is built (spec 39.2): `scripts/site.mjs`
// builds this file for Node, calls `render` for each language and writes
// what it returns into the built `index.html`. Nothing here runs in a
// visitor's browser.

import { renderToStaticMarkup } from "react-dom/server";
import { messagesOf, SITE_LOCALES, SITE_URL, type SiteLocale } from "./locales";
import { Page } from "./Page";

export { SITE_LOCALES, SITE_URL };

export interface Rendered {
  /** The `lang` of the page. */
  lang: SiteLocale;
  /** Where its file goes, inside the built site. */
  file: string;
  /** What goes in `<head>`: the title, the description, the other languages and the link previews. */
  head: string;
  /** The page itself. */
  body: string;
}

const ESCAPES: Record<string, string> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, (char) => ESCAPES[char] ?? char);
}

function tag(name: string, attributes: Record<string, string>): string {
  const pairs = Object.entries(attributes).map(([key, value]) => `${key}="${escapeHtml(value)}"`);
  return `<${name} ${pairs.join(" ")} />`;
}

/** The link preview's picture, in the page's language (made from the page, public-site). */
function pictureOf(locale: SiteLocale): string {
  return locale === "en" ? "og.png" : `og-${locale}.png`;
}

export function headOf(locale: SiteLocale): string {
  const t = messagesOf(locale);
  const own = SITE_LOCALES.find((each) => each.id === locale) ?? SITE_LOCALES[0];
  const url = `${SITE_URL}${own?.path ?? "/"}`;
  const lines = [
    `<title>${escapeHtml(t.meta.title)}</title>`,
    tag("meta", { name: "description", content: t.meta.description }),
    tag("link", { rel: "canonical", href: url }),
    ...SITE_LOCALES.map((each) =>
      tag("link", { rel: "alternate", hreflang: each.id, href: `${SITE_URL}${each.path}` }),
    ),
    tag("link", { rel: "alternate", hreflang: "x-default", href: `${SITE_URL}/` }),
    tag("meta", { property: "og:type", content: "website" }),
    tag("meta", { property: "og:site_name", content: "Botloft" }),
    tag("meta", { property: "og:title", content: t.meta.title }),
    tag("meta", { property: "og:description", content: t.meta.description }),
    tag("meta", { property: "og:url", content: url }),
    tag("meta", { property: "og:image", content: `${SITE_URL}/${pictureOf(locale)}` }),
    tag("meta", { property: "og:image:width", content: "1200" }),
    tag("meta", { property: "og:image:height", content: "630" }),
    tag("meta", { property: "og:locale", content: own?.og ?? "en_US" }),
    tag("meta", { name: "twitter:card", content: "summary_large_image" }),
  ];
  return lines.join("\n    ");
}

export function render(locale: SiteLocale): Rendered {
  const own = SITE_LOCALES.find((each) => each.id === locale);
  const folder = (own?.path ?? "/").replace(/^\//, "");
  return {
    lang: locale,
    file: `${folder}index.html`,
    head: headOf(locale),
    body: renderToStaticMarkup(<Page locale={locale} />),
  };
}
