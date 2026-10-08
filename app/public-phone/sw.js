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
