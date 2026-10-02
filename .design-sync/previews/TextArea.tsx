import { setLocaleChoice, TextArea } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function CrewGoal() {
  const [goal, setGoal] = useState(
    "Follow what's new in on-device AI every week and send me a short summary on Fridays.",
  );
  return (
    <div className="max-w-md p-4">
      <TextArea
        label="What is this crew for?"
        rows={4}
        value={goal}
        onChange={(event) => setGoal(event.target.value)}
        max={600}
        hint="The Chief reads this to plan the work and suggest bots."
      />
    </div>
  );
}

export function Empty() {
  return (
    <div className="max-w-md p-4">
      <TextArea
        label="Instructions"
        rows={3}
        placeholder="How Scout should work: sources it trusts, what to skip, when to ask you."
      />
    </div>
  );
}
