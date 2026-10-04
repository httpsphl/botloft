import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeHost } from "../../lib/fakeHost";
import { HostProvider } from "../../store/context";
import { Markdown } from "./Markdown";
import { splitMentions } from "./mentions";

afterEach(cleanup);

const mentions = (text: string) =>
  splitMentions(text)
    .filter((part) => part.mention)
    .map((part) => part.text);

describe("mentions", () => {
  test("are handles on their own, not e-mails or paths", () => {
    expect(mentions("@manychat confirmed, ask @atendente-2.")).toEqual([
      "@manychat",
      "@atendente-2",
    ]);
    expect(mentions("(@writer) and @Writer")).toEqual(["@writer", "@Writer"]);
    expect(mentions("mail ana@example.com or see C:\\x\\@cache and node_modules/@types")).toEqual(
      [],
    );
    expect(splitMentions("hi @scout!").map((part) => part.text)).toEqual(["hi ", "@scout", "!"]);
  });

  test("stand out in a reply, but not inside code or links", () => {
    const { container } = render(
      <HostProvider host={new FakeHost()}>
        <Markdown
          text={
            "O **@manychat** confirmou. Rode `npm i @types/x` e veja [@docs](https://example.com)."
          }
        />
      </HostProvider>,
    );
    const marked = [...container.querySelectorAll(".chat-mention")].map((node) => node.textContent);
    expect(marked).toEqual(["@manychat"]);
  });
});
