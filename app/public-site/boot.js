// Runs before the site's page shows (spec 39.2): the theme the system asks
// for, so the page never flashes the other one, and, on the English page,
// the language the visitor picked before or their browser's own. A plain
// file, since the page's policy allows no script written in the page.
(() => {
  const root = document.documentElement;
  root.dataset.theme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  if (window.location.pathname !== "/") return;
  const pages = { "pt-BR": "/pt-BR/", es: "/es/" };
  let wanted = null;
  try {
    wanted = localStorage.getItem("botloft.site.locale");
  } catch {
    // No storage: the browser's language decides.
  }
  if (!wanted) {
    for (const tag of navigator.languages || [navigator.language]) {
      const lower = String(tag).toLowerCase();
      if (lower.startsWith("pt")) wanted = "pt-BR";
      else if (lower.startsWith("es")) wanted = "es";
      else if (lower.startsWith("en")) wanted = "en";
      if (wanted) break;
    }
  }
  const page = pages[wanted];
  if (page) window.location.replace(page + window.location.hash);
})();
