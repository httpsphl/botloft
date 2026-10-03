// The fake daemon's `chat.search` (spec 8.8): the words, without accents or
// case, must all be in what was said to a bot, what it replied or its
// questions; the snippet marks each word found.

import type { FakeBotloft, Handlers } from "./fake";
import { invalid } from "./fakeRules";
import { type ChatBody, type SearchHit, SNIPPET_MARKS } from "./protocol.gen";

/** Lowercase and without accents, one character for each of `text`'s. */
function fold(text: string): string {
  return [...text].map((c) => c.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase()).join("");
}

function textOf(body: ChatBody): string | null {
  switch (body.kind) {
    case "inbound":
      return body.message.body;
    case "reply":
      return body.text;
    case "question":
      return `${body.question.text}\n${body.question.answer ?? ""}`;
    default:
      return null;
  }
}

/** The text on one line with every word found between the marks. */
function snippet(text: string, words: string[]): string {
  const flat = [...text.split(/\s+/).filter(Boolean).join(" ")];
  const folded = flat.map((c) => fold(c));
  const marked = new Array<boolean>(flat.length).fill(false);
  for (const word of words) {
    for (let at = 0; at + word.length <= flat.length; at += 1) {
      if (folded.slice(at, at + word.length).join("") === word) {
        marked.fill(true, at, at + word.length);
      }
    }
  }
  let out = "";
  flat.forEach((c, index) => {
    if (marked[index] && !marked[index - 1]) {
      out += SNIPPET_MARKS.open;
    }
    out += c;
    if (marked[index] && !marked[index + 1]) {
      out += SNIPPET_MARKS.close;
    }
  });
  return out;
}

export function searchHandlers(fake: FakeBotloft): Pick<Handlers, "chat.search"> {
  return {
    "chat.search": ({ query, botId, crewId, before, limit }) => {
      const words = fold(query)
        .split(/\s+/)
        .filter((word) => /[\p{L}\p{N}]/u.test(word));
      if (words.join("").replace(/[^\p{L}\p{N}]/gu, "").length < 2) {
        throw invalid("query needs at least 2 letters or digits");
      }
      const items = fake.chat.items;
      const end = before ? items.findIndex((item) => item.id === before) : items.length;
      const hits: SearchHit[] = [];
      for (const item of items.slice(0, end < 0 ? items.length : end).reverse()) {
        const bot = fake.bots.get(item.botId);
        const crew = bot && fake.crews.get(bot.crewId);
        const text = textOf(item.body);
        if (!bot || !crew || bot.archivedAt !== null || crew.archivedAt !== null || text === null) {
          continue;
        }
        if ((botId && bot.id !== botId) || (crewId && crew.id !== crewId)) {
          continue;
        }
        const folded = fold(text);
        if (words.every((word) => folded.includes(word))) {
          hits.push({ item, crewId: crew.id, snippet: snippet(text, words) });
        }
      }
      return hits.slice(0, limit ?? 30);
    },
  };
}
