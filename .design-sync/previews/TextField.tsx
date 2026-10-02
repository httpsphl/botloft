import { setLocaleChoice, TextField } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function CrewName() {
  const [name, setName] = useState("Research");
  return (
    <div className="max-w-sm p-4">
      <TextField
        label="Crew name"
        value={name}
        onChange={(event) => setName(event.target.value)}
        max={40}
        hint="You can change it later."
      />
    </div>
  );
}

export function Placeholder() {
  return (
    <div className="max-w-sm p-4">
      <TextField label="Bot name" placeholder="Scout, Writer, Ops…" />
    </div>
  );
}

export function OverLimit() {
  return (
    <div className="max-w-sm p-4">
      <TextField
        label="Crew name"
        value="The research crew that reads every paper about it"
        onChange={() => {}}
        max={40}
      />
    </div>
  );
}
