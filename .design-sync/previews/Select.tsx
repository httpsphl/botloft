import { Select, setLocaleChoice } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Language() {
  const [language, setLanguage] = useState("system");
  return (
    <div className="flex max-w-xs flex-col gap-1.5 p-4">
      <span className="font-medium text-ink-soft text-sm">Language</span>
      <Select
        label="Language"
        value={language}
        onChange={setLanguage}
        options={[
          { value: "system", label: "System (English)" },
          { value: "en", label: "English" },
          { value: "pt-BR", label: "Português (Brasil)" },
          { value: "es", label: "Español" },
        ]}
      />
    </div>
  );
}

export function Effort() {
  const [effort, setEffort] = useState("medium");
  return (
    <div className="flex max-w-xs flex-col gap-1.5 p-4">
      <span className="font-medium text-ink-soft text-sm">Effort</span>
      <Select
        label="Effort"
        value={effort}
        onChange={setEffort}
        options={[
          { value: "low", label: "Low" },
          { value: "medium", label: "Medium" },
          { value: "high", label: "High" },
        ]}
      />
    </div>
  );
}
