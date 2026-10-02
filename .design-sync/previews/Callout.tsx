import { Button, Callout, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Info() {
  return (
    <div className="max-w-md p-4">
      <Callout title="Your bots keep working after you close Botloft">
        They pick up where they left off after a restart, too.
      </Callout>
    </div>
  );
}

export function Warn() {
  return (
    <div className="max-w-md p-4">
      <Callout tone="warn" title="Scout reached its usage limit">
        It continues on its own at 4:10 PM.
      </Callout>
    </div>
  );
}

export function DangerWithAction() {
  return (
    <div className="max-w-md p-4">
      <Callout
        tone="danger"
        title="Claude Code is signed out"
        action={
          <Button size="sm" variant="primary">
            Sign in
          </Button>
        }
      >
        Your bots stopped until you sign in again.
      </Callout>
    </div>
  );
}

export function TitleOnly() {
  return (
    <div className="max-w-md p-4">
      <Callout title="Nothing is archived." />
    </div>
  );
}
