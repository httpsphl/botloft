// A message's Markdown as plain words, for one-line previews (the sidebar):
// the chat renders the marks, a preview line would show them raw, as in
// "**Sales of the day:** 1 sale".

/** The marks around a span of text, in the order they are taken off. */
const SPANS: [RegExp, string][] = [
  // Images and links keep their text: ![alt](url), [text](url).
  [/!\[([^\]]*)\]\([^)]*\)/g, "$1"],
  [/\[([^\]]+)\]\([^)]*\)/g, "$1"],
  // Code, then strong, emphasis and strikethrough. A lone _ or * inside a
  // word (snake_case, 2*3) stays.
  [/`([^`]+)`/g, "$1"],
  [/(\*\*|__)(?=\S)(.+?)(?<=\S)\1/g, "$2"],
  [/(^|[^\w*])\*(?=\S)(.+?)(?<=\S)\*(?![\w*])/g, "$1$2"],
  [/(^|[^\w_])_(?=\S)(.+?)(?<=\S)_(?![\w_])/g, "$1$2"],
  [/~~(?=\S)(.+?)(?<=\S)~~/g, "$1"],
];

/** The marks at the start of a line: headings, quotes, list items, rules. */
const LINE_START = /^\s{0,3}(?:#{1,6}\s+|>\s?|[-*+]\s+(?:\[[ xX]\]\s+)?|\d+[.)]\s+)/;

/** `markdown` as plain words on one line. */
export function plainText(markdown: string): string {
  const lines = markdown
    .split(/\r?\n/)
    .filter((line) => !/^\s*(?:```|~~~|[-*_]{3,}\s*$|\|?\s*:?-{3,})/.test(line))
    .map((line) => {
      let text = line;
      // Quotes and lists can nest: "> - item".
      for (let pass = 0; pass < 3 && LINE_START.test(text); pass++) {
        text = text.replace(LINE_START, "");
      }
      return text.replace(/\|/g, " ");
    });
  let text = lines.join(" ");
  for (const [mark, keep] of SPANS) {
    text = text.replace(mark, keep);
  }
  return text
    .replace(/\\([\\`*_{}[\]()#+\-.!>~|])/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
}
