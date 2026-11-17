import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeHost } from "../../lib/fakeHost";
import { HostProvider } from "../../store/context";
import { Markdown } from "./Markdown";

afterEach(cleanup);

const titles = (text: string) => {
  const { container } = render(
    <HostProvider host={new FakeHost()}>
      <Markdown text={text} />
    </HostProvider>,
  );
  return [...container.querySelectorAll("p")].map((p) => [
    p.textContent,
    p.hasAttribute("data-title"),
  ]);
};

describe("a reply's titles", () => {
  test("a line of bold words only opens a section, like a title", () => {
    expect(
      titles("**What I checked**\n\nThe log.\n\n**Next steps:**\n\n**Note:** nothing changed yet."),
    ).toEqual([
      ["What I checked", true],
      ["The log.", false],
      ["Next steps:", true],
      ["Note: nothing changed yet.", false],
    ]);
  });

  test("bold inside a sentence, or italic alone, is not a title", () => {
    expect(titles("It was **the cache**.\n\n*Maybe*\n\n**one** and **two**")).toEqual([
      ["It was the cache.", false],
      ["Maybe", false],
      ["one and two", false],
    ]);
  });
});
