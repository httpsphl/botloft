// What bots do with their tools, in plain words: the chat's tool lines, the
// requests to allow one and the conversation list. Keyed by the tool's name
// (the part after `mcp__botloft__` for ours); each is an action, so it
// reads right while it runs and after.

export const tools = {
  names: {
    Bash: "Run a command",
    PowerShell: "Run a command",
    BashOutput: "Read a command's output",
    KillShell: "Stop a command",
    KillBash: "Stop a command",
    Read: "Read a file",
    Write: "Write a file",
    Edit: "Edit a file",
    MultiEdit: "Edit a file",
    NotebookEdit: "Edit a notebook",
    Grep: "Search in files",
    Glob: "Find files",
    LS: "List a folder",
    WebFetch: "Read a web page",
    WebSearch: "Search the web",
    TodoWrite: "Update the to-do list",
    ExitPlanMode: "Present the plan",
    Task: "Call a helper",
    Agent: "Call a helper",
    ToolSearch: "Load tools",
    SlashCommand: "Run a shortcut",
    Skill: "Use a skill",
    crew_roster: "See the crew",
    send_message: "Send a message",
    complete_task: "Finish a task",
    my_tasks: "See its tasks",
    suggest_bot: "Suggest a bot",
    schedule_routine: "Set up a routine",
    browser: "Use a site",
    browser_help: "Ask for your hand",
    browser_open: "Open a page",
    browser_look: "Read the page",
    browser_click: "Click",
    browser_type: "Type in a field",
    browser_select: "Choose an option",
    browser_press: "Press a key",
    browser_scroll: "Scroll the page",
    browser_back: "Go back a page",
    browser_screenshot: "Look at the screen",
    browser_close: "Close the browser",
    browser_ask_owner: "Ask for your hand",
  },
  /** Where `browser_scroll` went. */
  scroll: { down: "down", up: "up", top: "to the top", bottom: "to the end" },
  /** A tool with no name of its own here, as an action. */
  use: (tool: string) => `use ${tool}`,
};
