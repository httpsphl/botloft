import { notifyError, setLocaleChoice, Toaster } from "@botloft/ui";
import { useEffect } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

// Toaster shows the notices raised with notifyError; it sits fixed at the
// bottom right of the window. The transform makes this box its containing
// block, so the card shows it.
const stage = { position: "relative", height: 160, transform: "translateZ(0)" } as const;

let raised = false;

export function ErrorNotice() {
  useEffect(() => {
    if (!raised) {
      raised = true;
      notifyError("Could not restart Scout", new Error("the bot's folder is in use by another program"));
    }
  }, []);
  return (
    <div style={stage}>
      <Toaster />
    </div>
  );
}
