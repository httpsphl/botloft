import { Button, Dialog, setLocaleChoice, TextArea, TextField } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

// The dialog covers the window (position: fixed). The transform makes this
// box its containing block, so the card shows it at a real size.
const stage = { position: "relative", height: 460, transform: "translateZ(0)" } as const;

export function EditBot() {
  return (
    <div style={stage}>
      <Dialog
        title="Edit Scout"
        onClose={() => {}}
        footer={
          <>
            <Button variant="ghost">Cancel</Button>
            <Button variant="primary">Save</Button>
          </>
        }
      >
        <div className="flex flex-col gap-4">
          <TextField label="Name" defaultValue="Scout" max={40} />
          <TextArea
            label="Role"
            rows={3}
            defaultValue="Finds sources for the Research crew and passes the best ones to Writer."
            hint="The other bots read this to know what to ask Scout."
          />
        </div>
      </Dialog>
    </div>
  );
}

export function Wide() {
  return (
    <div style={stage}>
      <Dialog
        title="What changed in Botloft 0.8.0"
        width="lg"
        onClose={() => {}}
        footer={<Button variant="primary">Got it</Button>}
      >
        <div className="flex flex-col gap-2 text-ink-soft text-sm">
          <p>Botloft now runs on Linux and macOS, as well as Windows.</p>
          <p>"Allow always" on a request keeps it as a rule of the bot, so it stops asking.</p>
          <p>Bots that reply while you are away leave a mark on the icon.</p>
        </div>
      </Dialog>
    </div>
  );
}
