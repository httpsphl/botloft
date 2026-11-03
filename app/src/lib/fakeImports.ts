// The fake daemon's history imports (spec 36.5, 36.6, 36.7): three Claude
// Code projects to choose from, a claude.ai export of 128 conversations in
// any ZIP named `export.zip`, a summary made from what was chosen, and
// sending it to the Maker as a file of an owner's message.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type {
  Import,
  ImportConversations,
  ImportDraft,
  ImportId,
  ImportProject,
  ImportsCreateParams,
} from "./protocol.gen";

const DAY_MS = 24 * 60 * 60 * 1000;

/** What the fake history holds. */
export const FAKE_PROJECTS: ImportProject[] = [
  {
    key: "C--work-blog",
    path: "C:\\work\\blog",
    sessions: 42,
    prompts: 318,
    firstAt: Date.UTC(2026, 9, 1),
    lastAt: Date.UTC(2026, 10, 3),
  },
  {
    key: "C--work-clinic",
    path: "C:\\work\\clinic",
    sessions: 17,
    prompts: 96,
    firstAt: Date.UTC(2026, 9, 10),
    lastAt: Date.UTC(2026, 10, 1),
  },
  {
    key: "C--work-site",
    path: "C:\\work\\site",
    sessions: 5,
    prompts: 12,
    firstAt: Date.UTC(2026, 9, 20),
    lastAt: Date.UTC(2026, 9, 28),
  },
];

/** What the fake claude.ai export holds. */
export const FAKE_EXPORT: ImportConversations = {
  count: 128,
  firstAt: Date.UTC(2025, 10, 4),
  lastAt: Date.UTC(2026, 10, 2),
  projects: ["Clinic", "Newsletter"],
};

function summary(projects: ImportProject[]): string {
  const sections = projects.map(
    (project) =>
      `## ${project.path}\n\n${project.sessions} sessions · ${project.prompts} requests\n\n` +
      "### Requests (repeated ones together, newest first)\n\n" +
      "- Summarize today's e-mails — 14 times, Mon/Wed/Fri ~9h\n" +
      "- Write the weekly newsletter with [key] — 4 times, Fri ~16h\n",
  );
  return `# Summary of my Claude Code history\n\nOnly the requests I typed.\n\n${sections.join("\n")}`;
}

export class FakeImports {
  readonly made: Import[] = [];
  private readonly texts = new Map<ImportId, string>();

  constructor(private readonly fake: FakeBotloft) {}

  /** The Claude Code projects asked for, or one stand-in for a claude.ai export. */
  private chosen({ source, keys, path }: ImportsCreateParams): ImportProject[] {
    if (source === "claude_ai") {
      if (!path?.endsWith("export.zip")) {
        throw invalid("choose the ZIP of your claude.ai export");
      }
      return [FAKE_PROJECTS[0] as ImportProject];
    }
    const chosen = FAKE_PROJECTS.filter((project) => keys?.includes(project.key));
    if (chosen.length === 0) {
      throw invalid("choose at least one project");
    }
    return chosen;
  }

  handlers(): Pick<
    Handlers,
    "imports.scan" | "imports.create" | "imports.send" | "imports.forget" | "imports.list"
  > {
    const unsent = (importId: ImportId) => {
      const made = this.made.find((each) => each.id === importId);
      if (!made || !this.texts.has(importId)) {
        throw notFound(`summary ${importId}`);
      }
      if (made.sentAt !== null) {
        throw conflict(`summary ${importId} was sent already`);
      }
      return made;
    };
    return {
      "imports.scan": ({ source, path }) => {
        if (source === "claude_code") {
          return { source, projects: FAKE_PROJECTS, conversations: null };
        }
        if (!path?.endsWith("export.zip")) {
          throw invalid("this is not an export of claude.ai: it has no conversations.json");
        }
        return { source, projects: [], conversations: FAKE_EXPORT };
      },
      "imports.create": (params): ImportDraft => {
        const { source } = params;
        const chosen = this.chosen(params);
        const text =
          source === "claude_code"
            ? summary(chosen)
            : "# Summary of my claude.ai conversations\n\n## Requests that repeat across conversations\n\n- Draft the clinic's weekly schedule — 9 times, Mon ~8h\n";
        const id = this.fake.id("imp") as ImportId;
        const period =
          source === "claude_code"
            ? {
                from: Math.min(...chosen.map((project) => project.firstAt ?? this.fake.now)),
                to: Math.max(...chosen.map((project) => project.lastAt ?? this.fake.now)),
              }
            : { from: params.from ?? FAKE_EXPORT.firstAt, to: FAKE_EXPORT.lastAt };
        this.texts.set(id, text);
        this.made.unshift({
          id,
          source,
          createdAt: this.fake.now,
          sentAt: null,
          sentMessageId: null,
          scope: source === "claude_code" ? chosen.map((project) => project.path) : ["claude.ai"],
          chars: text.length,
          redactions: { key: chosen.length, email: 1 },
        });
        return {
          importId: id,
          text,
          chars: text.length,
          tokensEstimate: Math.round(text.length / 3.6),
          cut: [],
          redactions: { key: chosen.length, email: 1 },
          ...period,
        };
      },
      "imports.send": ({ importId }) => {
        const made = unsent(importId);
        const text = this.texts.get(importId) ?? "";
        const fromCode = made.source === "claude_code";
        const message = this.fake.conversation.handlers()["messages.send"]({
          botId: this.fake.maker.ensure().id,
          body: fromCode
            ? "This is a summary of my Claude Code history. Suggest crews, routines and flows for me from it."
            : "This is a summary of my claude.ai conversations. Suggest crews, routines and flows for me from it.",
          attachments: [
            {
              name: fromCode ? "claude-code-history.md" : "claude-ai-history.md",
              mediaType: "text/markdown",
              data: btoa(String.fromCharCode(...new TextEncoder().encode(text))),
            },
          ],
        });
        made.sentAt = this.fake.now;
        made.sentMessageId = message.id;
        this.texts.delete(importId);
        return { messageId: message.id };
      },
      "imports.forget": ({ importId }) => {
        unsent(importId);
        this.texts.delete(importId);
        this.made.splice(
          this.made.findIndex((each) => each.id === importId),
          1,
        );
        return null;
      },
      "imports.list": () =>
        this.made.filter((each) => each.sentAt !== null || this.fake.now - each.createdAt < DAY_MS),
    };
  }
}
