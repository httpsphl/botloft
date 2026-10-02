// The fake daemon's "Allow always" (spec 10.1): which requests can be
// allowed for good, the rules kept per bot, and `rules.*`. The real daemon
// also allows a request a rule covers without asking; nothing here asks on
// its own, so that part is not played.

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { AllowRule, AllowScope, BotId } from "./protocol.gen";

type RuleMethods = Extract<keyof Handlers, `rules.${string}`>;

/** What "Allow always" covers for a request, as the daemon words it. */
export function scopeOf(toolName: string, input: string): AllowScope | null {
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(input) as Record<string, unknown>;
  } catch {
    return null;
  }
  const field = (name: string) => {
    const value = parsed[name];
    return typeof value === "string" && value.trim() ? value.trim() : null;
  };
  const scope = (kind: AllowScope["kind"], value: string | null): AllowScope | null =>
    value === null ? null : { toolName, kind, value };
  switch (toolName) {
    case "Bash":
    case "PowerShell":
      return scope("command", field("command"));
    case "WebFetch": {
      const url = field("url");
      try {
        return scope("site", url && new URL(url).hostname.replace(/^www\./, ""));
      } catch {
        return null;
      }
    }
    case "Read":
    case "Write":
    case "Edit":
    case "MultiEdit":
      return scope("file", field("file_path"));
    case "NotebookEdit":
      return scope("file", field("notebook_path"));
    case "WebSearch":
      return scope("tool", "");
    default:
      return null;
  }
}

export class FakeAllow {
  readonly rules: AllowRule[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** Keeps `scope` for the bot, once, and tells the app. */
  remember(botId: BotId, scope: AllowScope): void {
    const same = this.rules.some(
      (rule) =>
        rule.botId === botId &&
        rule.scope.toolName === scope.toolName &&
        rule.scope.kind === scope.kind &&
        rule.scope.value === scope.value,
    );
    if (!same) {
      this.rules.push({ id: this.fake.id("rul"), botId, scope, createdAt: this.fake.now });
    }
    this.fake.emit({ name: "bot.rules", params: { botId, rules: this.of(botId) } });
  }

  private of(botId: BotId): AllowRule[] {
    return this.rules.filter((rule) => rule.botId === botId);
  }

  handlers(): Pick<Handlers, RuleMethods> {
    return {
      "rules.list": ({ botId }) => {
        this.fake.bot(botId);
        return this.of(botId);
      },
      "rules.delete": ({ ruleId }) => {
        const at = this.rules.findIndex((rule) => rule.id === ruleId);
        const rule = this.rules[at];
        if (!rule) {
          throw notFound(`rule ${ruleId}`);
        }
        this.rules.splice(at, 1);
        const left = { botId: rule.botId, rules: this.of(rule.botId) };
        this.fake.emit({ name: "bot.rules", params: left });
        return left;
      },
    };
  }
}
