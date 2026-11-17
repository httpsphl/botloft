// Builds the public site (spec 39) into dist-site, which the account server
// serves at /. First the page's script and styles for the browser, then the
// page drawn in Node for each language (src/site/render.tsx), written into
// a copy of the built index.html: /, /pt-BR/ and /es/. The site works and
// reads whole before any script runs.
//
//   pnpm build:site

import { copyFile, mkdir, readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { dirname, extname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "vite";

const app = fileURLToPath(new URL("..", import.meta.url));
const configFile = join(app, "vite.config.ts");
const out = join(app, "dist-site");
const drawn = join(app, "dist-site-ssr");

// 1. What the browser loads: the script that wakes the mascots, and the styles.
await build({ configFile, mode: "site", logLevel: "warn" });

// 2. The page itself, for Node. Its pictures come out with the same names
//    the page links to, and join the browser's files.
await build({
  configFile,
  mode: "site",
  logLevel: "warn",
  build: {
    ssr: "render.tsx",
    outDir: drawn,
    emptyOutDir: true,
    ssrEmitAssets: true,
    copyPublicDir: false,
  },
  // GSAP's package is CommonJS for Node: bundle it rather than load it.
  ssr: { noExternal: ["gsap"] },
});
const pictures = join(drawn, "assets");
for (const name of await readdir(pictures).catch(() => [])) {
  if (![".js", ".mjs", ".map", ".css"].includes(extname(name))) {
    await copyFile(join(pictures, name), join(out, "assets", name));
  }
}

// 3. A page per language.
const { render, SITE_LOCALES, SITE_URL } = await import(
  pathToFileURL(join(drawn, "render.js")).href
);
const template = await readFile(join(out, "index.html"), "utf8");
for (const marker of ['<html lang="en"', "<!--head-->", "<!--page-->"]) {
  if (!template.includes(marker)) {
    throw new Error(`the built index.html has no ${marker}`);
  }
}
for (const { id } of SITE_LOCALES) {
  const page = render(id);
  // Functions, so a "$" in the text is never read as a pattern.
  const html = template
    .replace('<html lang="en"', () => `<html lang="${page.lang}"`)
    .replace("<!--head-->", () => page.head)
    .replace("<!--page-->", () => page.body);
  const file = join(out, page.file);
  await mkdir(dirname(file), { recursive: true });
  await writeFile(file, html);
}

// 4. The app's icon, the same as the phone page's.
for (const name of ["icon-192.png", "apple-touch-icon.png"]) {
  await copyFile(join(app, "public-phone", name), join(out, name));
}

// 5. For search engines: the three pages, and what is not the site.
const urls = SITE_LOCALES.map(({ path }) => {
  const alternates = SITE_LOCALES.map(
    (other) =>
      `    <xhtml:link rel="alternate" hreflang="${other.id}" href="${SITE_URL}${other.path}"/>`,
  ).join("\n");
  return `  <url>\n    <loc>${SITE_URL}${path}</loc>\n${alternates}\n  </url>`;
}).join("\n");
await writeFile(
  join(out, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n${urls}\n</urlset>\n`,
);
await writeFile(
  join(out, "robots.txt"),
  `User-agent: *\nDisallow: /m/\nDisallow: /v1/\nDisallow: /download/\n\nSitemap: ${SITE_URL}/sitemap.xml\n`,
);

await rm(drawn, { recursive: true, force: true });

let total = 0;
async function size(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) await size(path);
    else total += (await stat(path)).size;
  }
}
await size(out);
console.log(
  `site: ${SITE_LOCALES.length} pages in dist-site, ${(total / 1024 / 1024).toFixed(1)} MB in all`,
);
