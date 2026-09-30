// A shell command a bot runs or asks to run (spec 10.1). Its input holds
// the command and, maybe, what the bot says it is for; the daemon sends
// that apart, as the item's `explanation`.

/** The tools that run a shell command. */
export function isCommand(tool: string): boolean {
  return tool === "Bash" || tool === "PowerShell";
}

export interface Command {
  /** The command as the bot wrote it, line breaks and all. */
  text: string;
  /** False when it was too long to keep and the end is missing. */
  whole: boolean;
}

/** The command in a call's input. One cut short does not parse: it is shown as it came. */
export function commandOf(input: string): Command {
  try {
    const parsed: unknown = JSON.parse(input);
    if (typeof parsed === "object" && parsed !== null && "command" in parsed) {
      const { command } = parsed;
      if (typeof command === "string") {
        return { text: command, whole: true };
      }
    }
    return { text: JSON.stringify(parsed, null, 2), whole: true };
  } catch {
    return { text: input, whole: false };
  }
}
