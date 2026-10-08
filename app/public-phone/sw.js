// The phone's page keeps its own files for the next visit, and nothing else:
// never an answer from the server and never what a bot asked (spec 28.7).
const CACHE = "botloft-phone-1";

self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) =>
        Promise.all(names.filter((name) => name !== CACHE).map((name) => caches.delete(name))),
      )
      .then(() => self.clients.claim()),
  );
});

// A push says only that something waits, with nothing in it (spec 28.8): the
// words are here, in the phone's language, and the details are in the page.
const WORDS = {
  en: ["Botloft needs you", "Something is waiting for you."],
  pt: ["O Botloft precisa de você", "Tem algo esperando você."],
  es: ["Botloft te necesita", "Hay algo esperando."],
};

function words() {
  const language = (self.navigator.language || "en").toLowerCase();
  if (language.startsWith("pt")) return WORDS.pt;
  if (language.startsWith("es")) return WORDS.es;
  return WORDS.en;
}

self.addEventListener("push", (event) => {
  const [title, body] = words();
  // One notice stands for all that wait; a new push brings it back up.
  event.waitUntil(
    self.registration.showNotification(title, {
      body,
      tag: "botloft-waiting",
      renotify: true,
      icon: "/m/icon-192.png",
      badge: "/m/icon-192.png",
    }),
  );
});

self.addEventListener("notificationclick", (event) => {
  event.notification.close();
  event.waitUntil(
    self.clients.matchAll({ type: "window", includeUncontrolled: true }).then((windows) => {
      const open = windows.find((window) => new URL(window.url).pathname.startsWith("/m"));
      return open ? open.focus() : self.clients.openWindow("/m/");
    }),
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  const url = new URL(request.url);
  if (
    request.method !== "GET" ||
    url.origin !== self.location.origin ||
    !url.pathname.startsWith("/m/")
  ) {
    return;
  }
  // The built files have a hash in their names: once kept, they never change.
  if (url.pathname.startsWith("/m/assets/")) {
    event.respondWith(
      caches.match(request).then(
        (kept) =>
          kept ||
          fetch(request).then((fresh) => {
            const copy = fresh.clone();
            caches.open(CACHE).then((cache) => cache.put(request, copy));
            return fresh;
          }),
      ),
    );
    return;
  }
  // The page itself comes from the server when it can, and from here when not.
  event.respondWith(
    fetch(request)
      .then((fresh) => {
        const copy = fresh.clone();
        caches.open(CACHE).then((cache) => cache.put(request, copy));
        return fresh;
      })
      .catch(() => caches.match(request).then((kept) => kept || caches.match("/m/"))),
  );
});
