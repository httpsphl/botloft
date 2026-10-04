// "Less motion" in Settings (spec 15.3): marks the root, which
// `motion-less.css` follows.

import { useEffect } from "react";
import { prefs, usePref } from "./prefs";

export function useMotionRoot(): void {
  const less = usePref(prefs.lessMotion);
  useEffect(() => {
    if (less) {
      document.documentElement.dataset.motion = "less";
    } else {
      delete document.documentElement.dataset.motion;
    }
  }, [less]);
}
