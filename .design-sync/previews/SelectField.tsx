import { SelectField, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function Model() {
  return (
    <div className="max-w-sm p-4">
      <SelectField
        label="Model"
        defaultValue="sonnet"
        hint="Opus thinks deeper; Haiku answers fastest."
        options={[
          { value: "opus", label: "Opus" },
          { value: "sonnet", label: "Sonnet" },
          { value: "haiku", label: "Haiku" },
        ]}
      />
    </div>
  );
}

export function Disabled() {
  return (
    <div className="max-w-sm p-4">
      <SelectField
        label="Crew"
        defaultValue="research"
        disabled
        options={[{ value: "research", label: "Research" }]}
      />
    </div>
  );
}
