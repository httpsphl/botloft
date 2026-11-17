// The public site (spec 39): each language drawn whole, what search engines
// and link previews read, the download offered first and the script that
// runs before the page shows.

import { describe, expect, it } from "vitest";
import boot from "../../public-site/boot.js?raw";
import { localeOfPath, messagesOf, SITE_LOCALES, type SiteLocale } from "./locales";
import { readMascot } from "./Mascot";
import { headOf, render } from "./render";
import { MAIN_DOWNLOAD, systemOf } from "./systems";

function drawn(locale: SiteLocale): HTMLElement {
  const holder = document.createElement("div");
  holder.innerHTML = render(locale).body;
  return holder;
}

describe("the site's pages", () => {
  it.each(SITE_LOCALES.map((each) => [each.id, each.path] as const))(
    "draws %s whole, at %s",
    (id, path) => {
      const page = render(id);
      const t = messagesOf(id);
      expect(page.lang).toBe(id);
      expect(page.file).toBe(`${path.slice(1)}index.html`);
      const holder = drawn(id);
      expect(holder.querySelector("h1")?.textContent).toBe(t.hero.title);
      expect(holder.querySelectorAll("section h2").length).toBeGreaterThanOrEqual(6);
      // Every card has its scene, and the opening has the crew at work.
      expect(holder.querySelectorAll(".site-demo-card [data-scene]").length).toBe(7);
      expect(holder.querySelector(".demo-hero")?.getAttribute("aria-label")).toBe(t.hero.demo);
      // The questions open without a script.
      expect(holder.querySelectorAll(".site-faq details").length).toBe(
        Object.keys(t.faq.items).length,
      );
      // The language switch marks this page, and leads to the other two.
      expect(holder.querySelector(`[data-locale="${id}"]`)?.getAttribute("aria-current")).toBe(
        "page",
      );
      expect(
        [...holder.querySelectorAll<HTMLAnchorElement>(".site-languages a")].map((a) =>
          a.getAttribute("href"),
        ),
      ).toEqual(expect.arrayContaining(["/", "/pt-BR/", "/es/"]));
    },
  );

  it("holds nothing the page's policy would refuse", () => {
    for (const { id } of SITE_LOCALES) {
      const { body } = render(id);
      expect(body).not.toMatch(/\sstyle=/);
      expect(body).not.toContain("<script");
      expect(body).not.toMatch(/\son[a-z]+=/);
    }
  });

  it("leaves a place for each mascot, with what to wake there", () => {
    const places = [...drawn("en").querySelectorAll<HTMLElement>(".site-mascot")];
    expect(places.length).toBeGreaterThan(10);
    for (const place of places) {
      const mascot = readMascot(place);
      expect(mascot).not.toBeNull();
      // The place keeps the mascot's size, so nothing moves when it wakes.
      expect(place.querySelector("svg")?.getAttribute("height")).toBe(String(mascot?.size));
    }
    expect(places.some((place) => place.dataset.mood === "working")).toBe(true);
  });

  it("offers every installer, and Windows first until the script knows better", () => {
    const holder = drawn("es");
    const links = [...holder.querySelectorAll<HTMLAnchorElement>("a")].map((a) =>
      a.getAttribute("href"),
    );
    for (const target of ["windows", "macos", "linux-deb", "linux-appimage"]) {
      expect(links).toContain(`/download/${target}`);
    }
    const shown = [...holder.querySelectorAll<HTMLElement>("[data-system]")].filter(
      (button) => !button.hidden,
    );
    expect(shown.map((button) => button.dataset.system)).toEqual(["windows", "windows"]);
  });

  it("tells search engines and link previews about the page and its languages", () => {
    const head = headOf("pt-BR");
    const t = messagesOf("pt-BR");
    expect(head).toContain(`<title>${t.meta.title}</title>`);
    expect(head).toContain(
      '<link rel="canonical" href="https://botloft.comitium.com.br/pt-BR/" />',
    );
    for (const { id, path } of SITE_LOCALES) {
      expect(head).toContain(
        `<link rel="alternate" hreflang="${id}" href="https://botloft.comitium.com.br${path}" />`,
      );
    }
    expect(head).toContain('hreflang="x-default"');
    expect(head).toContain('<meta property="og:locale" content="pt_BR" />');
    expect(head).toContain(
      '<meta property="og:image" content="https://botloft.comitium.com.br/og-pt-BR.png" />',
    );
    expect(headOf("en")).toContain('content="https://botloft.comitium.com.br/og.png"');
  });
});

describe("the site's languages", () => {
  it("are read from the page's address", () => {
    expect(localeOfPath("/")).toBe("en");
    expect(localeOfPath("/pt-BR/")).toBe("pt-BR");
    expect(localeOfPath("/pt-BR")).toBe("pt-BR");
    expect(localeOfPath("/es/")).toBe("es");
    expect(localeOfPath("/fr/")).toBe("en");
  });
});

describe("the download offered first", () => {
  const chrome = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/142.0";
  const safari =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Safari/605.1";
  const linux = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/142.0";
  const android =
    "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 Mobile Safari/537.36";
  const iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 19_0 like Mac OS X) Mobile/15E148";

  it("is the visitor's own system", () => {
    expect(systemOf(chrome, "Windows")).toBe("windows");
    expect(systemOf(safari, "MacIntel")).toBe("macos");
    expect(systemOf(linux, "Linux x86_64")).toBe("linux");
    expect(systemOf(android, "Android")).toBe("phone");
    expect(systemOf(iphone, "iPhone")).toBe("phone");
    // An iPad asking for the desktop site says it is a Mac.
    expect(systemOf(safari, "MacIntel", 5)).toBe("phone");
    expect(systemOf("", "")).toBe("windows");
  });

  it("sends a phone to the list instead of an installer", () => {
    expect(MAIN_DOWNLOAD.phone).toBe("#downloads");
    expect(MAIN_DOWNLOAD.linux).toBe("/download/linux-deb");
  });
});

describe("the script before the page shows", () => {
  function run(path: string, saved: string | null, languages: string[]) {
    const went: string[] = [];
    const page = {
      location: { pathname: path, hash: "#faq", replace: (to: string) => went.push(to) },
    };
    const storage = { getItem: () => saved };
    // The plain file the page loads, run with stand-ins for the browser.
    new Function("window", "localStorage", "navigator", boot)(page, storage, { languages });
    return went;
  }

  it("sends a first visit of the English page to the visitor's language", () => {
    expect(run("/", null, ["pt-BR", "en"])).toEqual(["/pt-BR/#faq"]);
    expect(run("/", null, ["es-AR"])).toEqual(["/es/#faq"]);
    expect(run("/", null, ["en-US", "pt-BR"])).toEqual([]);
    expect(run("/", null, ["fr-FR", "es"])).toEqual(["/es/#faq"]);
  });

  it("keeps a language the visitor picked, and never moves the other pages", () => {
    expect(run("/", "en", ["pt-BR"])).toEqual([]);
    expect(run("/", "es", ["pt-BR"])).toEqual(["/es/#faq"]);
    expect(run("/pt-BR/", null, ["es"])).toEqual([]);
  });
});
