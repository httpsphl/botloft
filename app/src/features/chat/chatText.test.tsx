import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeHost } from "../../lib/fakeHost";
import { HostProvider } from "../../store/context";
import { ChatText, splitLinks } from "./ChatText";

afterEach(cleanup);

const links = (text: string) =>
  splitLinks(text)
    .filter((part) => part.link)
    .map((part) => part.text);

describe("web addresses in plain chat text (spec 15.3)", () => {
  test("an address is a link, without what ends the sentence around it", () => {
    expect(links("See https://op.gg/pt/lol/summoners/br/some%20player-br1, please.")).toEqual([
      "https://op.gg/pt/lol/summoners/br/some%20player-br1",
    ]);
    expect(links("(the docs: https://example.com/a)")).toEqual(["https://example.com/a"]);
    expect(links("https://en.wikipedia.org/wiki/Kindred_(League)!")).toEqual([
      "https://en.wikipedia.org/wiki/Kindred_(League)",
    ]);
    expect(links("two: http://a.com and https://b.com/?q=1&x=2.")).toEqual([
      "http://a.com",
      "https://b.com/?q=1&x=2",
    ]);
  });

  test("text that only looks like one stays text", () => {
    expect(links("ftp://a.com, www.a.com, https:// and a@b.com")).toEqual([]);
    expect(
      splitLinks("go to https://a.com now")
        .map((part) => part.text)
        .join(""),
    ).toBe("go to https://a.com now");
  });

  test("the link opens in the browser, and mentions stay marked", () => {
    const host = new FakeHost();
    const { container } = render(
      <HostProvider host={host}>
        <p>
          <ChatText text="@scout look at https://op.gg/x." />
        </p>
      </HostProvider>,
    );
    expect(container.querySelector(".chat-mention")?.textContent).toBe("@scout");
    fireEvent.click(screen.getByRole("link", { name: "https://op.gg/x" }));
    expect(host.opened).toContain("https://op.gg/x");
    expect(container.textContent).toBe("@scout look at https://op.gg/x.");
  });
});
