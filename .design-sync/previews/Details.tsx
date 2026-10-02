import { Callout, Details, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Closed() {
  return (
    <div className="max-w-md p-4">
      <p className="text-ink text-sm">Scout could not start.</p>
      <Details>claude exited with code 1 after 2.3 s</Details>
    </div>
  );
}

export function Open() {
  return (
    <div className="max-w-md p-4">
      <p className="text-ink text-sm">Scout could not start.</p>
      <Details open label="What happened">
        {"claude exited with code 1 after 2.3 s\nNo conversation found with session ID 4f2c…"}
      </Details>
    </div>
  );
}

export function InCallout() {
  return (
    <div className="max-w-md p-4">
      <Callout tone="danger" title="The update could not be installed">
        Botloft keeps working on the current version.
        <Details>signature check failed for Botloft_0.8.1_x64-setup.exe</Details>
      </Callout>
    </div>
  );
}
