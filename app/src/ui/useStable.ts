// A callback that keeps its identity across renders but always runs the
// latest `fn`, so memoized children and context consumers do not re-render
// when the parent does. Not for use during render.

import { useCallback, useLayoutEffect, useRef } from "react";

export function useStable<A extends unknown[], R>(fn: (...args: A) => R): (...args: A) => R {
  const latest = useRef(fn);
  useLayoutEffect(() => {
    latest.current = fn;
  });
  return useCallback((...args: A) => latest.current(...args), []);
}
