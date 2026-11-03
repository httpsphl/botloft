// The shared site logins (spec 35), read when shown and kept current by
// `logins.changed`, with what the last window to close found.

import { useEffect, useState } from "react";
import type { LoginList } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const NONE: LoginList = { logins: [], window: null };

export interface Closed {
  site: string;
  found: boolean;
}

/** `list` is `null` until the daemon has answered; an older one has none. */
export function useLogins(): { list: LoginList | null; closed: Closed | null } {
  const api = useApi();
  const [list, setList] = useState<LoginList | null>(null);
  const [closed, setClosed] = useState<Closed | null>(null);
  useEffect(() => {
    let alive = true;
    const stop = api.subscribe((event) => {
      if (event.name !== "logins.changed") {
        return;
      }
      setList(event.params.list);
      const { closed, found } = event.params;
      if (closed !== null) {
        setClosed({ site: closed, found: found === true });
      } else if (event.params.list.window !== null) {
        setClosed(null);
      }
    });
    api.call("logins.list").then(
      (listed) => alive && setList((current) => current ?? listed),
      () => alive && setList((current) => current ?? NONE),
    );
    return () => {
      alive = false;
      stop();
    };
  }, [api]);
  return { list, closed };
}
