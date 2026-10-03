// Ctrl+K opens the search (spec 8.8), from anywhere in the app; with it
// already open, it selects what is in its field.

import { useEffect } from "react";
import { useApp } from "../../store/context";

export function useSearchKey(): void {
  const openPage = useApp((state) => state.openPage);
  const open = useApp((state) => state.page === "search");
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const command = event.ctrlKey || event.metaKey;
      if (!command || event.altKey || event.shiftKey || event.key.toLowerCase() !== "k") {
        return;
      }
      event.preventDefault();
      if (open) {
        // Selected, so typing starts a new search.
        document.querySelector<HTMLInputElement>('input[type="search"]')?.select();
      } else {
        openPage("search");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, openPage]);
}
