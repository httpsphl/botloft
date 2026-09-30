// Loading the app store from the daemon and following its notifications.

import { type BotloftApi, type ConnectionState, errorText } from "../lib/api";
import type { Delivery } from "../lib/protocol.gen";
import { type AppStore, applyEvent, crewList } from "./app";

/** How often `system.status` is refreshed; it has no notification. */
const STATUS_POLL_MS = 15_000;

/** Keeps `store` in sync with the daemon until the returned function runs. */
export function syncStore(store: AppStore, api: BotloftApi): () => void {
  let poll: ReturnType<typeof setInterval> | undefined;
  let alive = true;

  const refreshStatus = () => {
    api.call("system.status").then(
      (system) => alive && store.setState({ system }),
      () => {},
    );
  };

  // Each part is applied as soon as it arrives: a notification that comes
  // later in the stream is newer than the list and must win.
  const load = () => {
    store.setState({ loaded: false, loadError: null });
    const fail = (error: unknown) => alive && store.setState({ loadError: errorText(error) });
    refreshStatus();
    api.call("settings.get").then(
      (settings) => alive && store.setState({ settings }),
      // An older daemon has no settings: they stay unset.
      () => {},
    );
    const crews = api.call("crews.list").then((list) => {
      if (alive) {
        store.setState({ crews: Object.fromEntries(list.map((crew) => [crew.id, crew])) });
      }
    });
    const bots = api.call("bots.list", {}).then((list) => {
      if (alive) {
        store.setState({
          bots: Object.fromEntries(list.map((bot) => [bot.id, bot])),
          activity: Object.fromEntries(list.map((bot) => [bot.id, bot.lastActivity])),
        });
      }
    });
    // Recent deliveries plus every dead one: the dead need the owner.
    Promise.all([api.call("deliveries.list", {}), api.call("deliveries.list", { state: "dead" })])
      .then(([recent, dead]) => {
        if (alive) {
          const byMessage: Record<string, Delivery> = {};
          for (const delivery of [...dead, ...recent]) {
            byMessage[delivery.messageId] = delivery;
          }
          store.setState({ deliveries: byMessage });
        }
      })
      .catch(() => {});
    api
      .call("routines.list", {})
      .then(
        (list) =>
          alive && store.setState({ routines: Object.fromEntries(list.map((r) => [r.id, r])) }),
      )
      .catch(() => {});
    api
      .call("browser.list")
      .then(
        (list) =>
          alive && store.setState({ browsers: Object.fromEntries(list.map((b) => [b.botId, b])) }),
      )
      .catch(() => {});
    api
      .call("tasks.list", {})
      .then(
        (list) =>
          alive && store.setState({ tasks: Object.fromEntries(list.map((t) => [t.id, t])) }),
      )
      .catch(() => {});
    Promise.all([crews, bots]).then(() => {
      if (!alive) {
        return;
      }
      const state = store.getState();
      const valid = state.selectedCrewId !== null && state.crews[state.selectedCrewId];
      if (!valid) {
        state.selectCrew(crewList(state)[0]?.id ?? null);
      }
      store.setState({ loaded: true });
    }, fail);
  };

  const onConnection = (connection: ConnectionState) => {
    store.setState({ connection });
    clearInterval(poll);
    if (connection.kind === "open") {
      load();
      poll = setInterval(refreshStatus, STATUS_POLL_MS);
    }
  };

  const unsubscribeEvents = api.subscribe((event) => {
    const change = applyEvent(store.getState(), event);
    if (change) {
      store.setState(change);
    }
  });
  const unsubscribeConnection = api.onConnection(onConnection);
  onConnection(api.connection());

  return () => {
    alive = false;
    clearInterval(poll);
    unsubscribeEvents();
    unsubscribeConnection();
  };
}
