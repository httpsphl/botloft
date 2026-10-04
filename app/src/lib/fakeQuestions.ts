// The fake daemon's questions to the owner (spec 23): `ask` plays a bot
// calling `ask_owner`, and `questions.*` answer and dismiss them like the
// daemon does, the answer going to the bot as the owner's message.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type { BotId, ChatItem, Question } from "./protocol.gen";

type QuestionMethods = Extract<keyof Handlers, `questions.${string}`>;

export class FakeQuestions {
  /** Every question, oldest first, with the chat item that shows it. */
  readonly asked: { question: Question; itemId: string }[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** The bot asks the owner something; the app hears of it. */
  ask(botId: BotId, text: string, options: string[] = []): Question {
    const bot = this.fake.bot(botId);
    const question: Question = {
      id: this.fake.id("qst"),
      crewId: bot.crewId,
      botId,
      text,
      options,
      status: "open",
      answer: null,
      createdAt: this.fake.now,
      answeredAt: null,
    };
    const item: ChatItem = this.fake.chat.add(botId, { kind: "question", question });
    this.asked.push({ question, itemId: item.id });
    this.fake.emit({ name: "question.changed", params: question });
    return question;
  }

  private open(questionId: string): { question: Question; itemId: string } {
    const found = this.asked.find((entry) => entry.question.id === questionId);
    if (!found) {
      throw notFound(`question ${questionId}`);
    }
    if (found.question.status !== "open") {
      throw conflict(`question ${questionId} was already answered or dismissed`);
    }
    this.fake.bot(found.question.botId);
    return found;
  }

  private close(entry: { question: Question; itemId: string }, changes: Partial<Question>) {
    const question: Question = { ...entry.question, ...changes, answeredAt: this.fake.now };
    entry.question = question;
    this.fake.chat.update(entry.itemId, { kind: "question", question });
    this.fake.emit({ name: "question.changed", params: question });
    return question;
  }

  handlers(): Pick<Handlers, QuestionMethods> {
    return {
      "questions.list": ({ status }) => {
        const wanted = status ?? "open";
        return this.asked
          .map((entry) => entry.question)
          .filter((question) => {
            const bot = this.fake.bots.get(question.botId);
            const crew = this.fake.crews.get(question.crewId);
            return (
              question.status === wanted && bot?.archivedAt === null && crew?.archivedAt === null
            );
          })
          .reverse();
      },
      "questions.answer": ({ questionId, answer }) => {
        const body = answer.trim();
        if (!body) {
          throw invalid("answer must not be empty");
        }
        const entry = this.open(questionId);
        const question = this.close(entry, { status: "answered", answer: body });
        this.fake.conversation.record({
          id: this.fake.id("msg"),
          crewId: question.crewId,
          fromKind: "owner",
          fromBotId: null,
          toBotId: question.botId,
          kind: "note",
          body,
          taskId: null,
          routineId: null,
          questionId: question.id,
          attachments: [],
          replyTo: null,
          createdAt: this.fake.now,
        });
        return question;
      },
      "questions.dismiss": ({ questionId }) =>
        this.close(this.open(questionId), { status: "dismissed" }),
    };
  }
}
