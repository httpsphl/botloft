import { Choices, setLocaleChoice } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Model() {
  const [model, setModel] = useState("sonnet");
  return (
    <div className="flex flex-col gap-2 p-4">
      <span className="font-medium text-ink-soft text-sm">Model</span>
      <Choices
        label="Model"
        value={model}
        onChange={setModel}
        options={[
          { value: "opus", label: "Opus" },
          { value: "sonnet", label: "Sonnet" },
          { value: "haiku", label: "Haiku" },
        ]}
      />
    </div>
  );
}

export function WaitTime() {
  const [minutes, setMinutes] = useState(60);
  return (
    <div className="flex flex-col gap-2 p-4">
      <span className="font-medium text-ink-soft text-sm">How long a request waits for you</span>
      <Choices
        label="How long a request waits for you"
        value={minutes}
        onChange={setMinutes}
        options={[
          { value: 15, label: "15 min" },
          { value: 30, label: "30 min" },
          { value: 60, label: "1 h" },
          { value: 120, label: "2 h" },
          { value: 240, label: "4 h" },
          { value: 480, label: "8 h" },
        ]}
      />
    </div>
  );
}
