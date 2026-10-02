import { setLocaleChoice, Tabs } from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

export function BotTabs() {
  const [tab, setTab] = useState<"chat" | "tasks" | "routines">("chat");
  return (
    <div className="w-full max-w-lg bg-panel">
      <Tabs
        label="Scout"
        value={tab}
        onChange={setTab}
        tabs={[
          { id: "chat", label: "Chat" },
          { id: "tasks", label: "Tasks" },
          { id: "routines", label: "Routines" },
        ]}
      />
      <p className="p-4 text-muted text-sm">
        {tab === "chat" ? "Scout's conversation" : tab === "tasks" ? "3 open tasks" : "Every weekday at 9:00"}
      </p>
    </div>
  );
}

export function SecondSelected() {
  const [tab, setTab] = useState<"general" | "chat" | "alerts">("chat");
  return (
    <div className="w-full max-w-lg bg-panel">
      <Tabs
        label="Settings"
        value={tab}
        onChange={setTab}
        tabs={[
          { id: "general", label: "General" },
          { id: "chat", label: "Chat" },
          { id: "alerts", label: "Alerts" },
        ]}
      />
    </div>
  );
}
