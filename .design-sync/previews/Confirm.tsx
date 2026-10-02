import { Confirm, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

// Confirm opens a Dialog, which covers the window (position: fixed). The
// transform makes this box its containing block.
const stage = { position: "relative", height: 400, transform: "translateZ(0)" } as const;

export function ArchiveBot() {
  return (
    <div style={stage}>
      <Confirm
        title="Archive Scout?"
        confirmLabel="Archive"
        onConfirm={async () => {}}
        onClose={() => {}}
      >
        Scout stops now and leaves the sidebar. Botloft keeps its conversation, and you can delete it
        later in Settings &gt; Archived.
      </Confirm>
    </div>
  );
}
