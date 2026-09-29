import { act, cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { readMark } from "./cursorMark";
import { LiveFrame } from "./LiveFrame";

afterEach(cleanup);

const PAGE = { width: 1280, height: 800 };
const VIEW = PAGE;
const scout = { name: "Scout", color: "#ff6a3d" };

function draw(writer: typeof scout | null = scout) {
  const view = render(
    <LiveFrame
      url="data:text/html,<h1>Bak"
      device="desktop"
      scale={0.5}
      title="home.html"
      writer={writer}
    />,
  );
  const frame = view.container.querySelector("iframe") as HTMLIFrameElement;
  return { view, frame };
}

function say(data: unknown, source: Window | null) {
  act(() => {
    window.dispatchEvent(new MessageEvent("message", { data, source }));
  });
}

describe("the bot's cursor on a screen", () => {
  test("follows what the draft's script says, scaled to the artboard", () => {
    const { view, frame } = draw();
    expect(view.container.querySelector(".bot-cursor")).toBeNull();
    say(
      {
        botloft: "cursor",
        view: VIEW,
        point: { x: 200, y: 120 },
        box: { x: 64, y: 100, width: 400, height: 40 },
      },
      frame.contentWindow,
    );
    const cursor = view.container.querySelector(".bot-cursor") as HTMLElement;
    expect(cursor.style.left).toBe("100px");
    expect(cursor.style.top).toBe("60px");
    expect(cursor.textContent).toContain("Scout");
    const box = cursor.previousElementSibling as HTMLElement;
    expect(box.style.left).toBe(`${64 * 0.5 - 3}px`);
    expect(box.style.width).toBe(`${400 * 0.5 + 6}px`);

    // Nothing on the page yet: it waits at the top.
    say({ botloft: "cursor", view: VIEW, point: null, box: null }, frame.contentWindow);
    expect((view.container.querySelector(".bot-cursor") as HTMLElement).style.left).toBe("20px");
  });

  test("the next version's mark shows once that version is on show", () => {
    const { view } = draw();
    const next = (url: string) => (
      <LiveFrame url={url} device="desktop" scale={0.5} title="home.html" writer={scout} />
    );
    view.rerender(next("data:text/html,<h1>Bakery"));
    const [, back] = view.container.querySelectorAll("iframe");
    say(
      { botloft: "cursor", view: VIEW, point: { x: 400, y: 200 }, box: null },
      (back as HTMLIFrameElement).contentWindow,
    );
    expect(view.container.querySelector(".bot-cursor")).toBeNull();

    act(() => {
      back?.dispatchEvent(new Event("load"));
    });
    const cursor = view.container.querySelector(".bot-cursor") as HTMLElement;
    expect(cursor.style.left).toBe("200px");
  });

  test("hears only its own frames, and only while the bot writes", () => {
    const { view, frame } = draw();
    say({ botloft: "cursor", view: VIEW, point: { x: 10, y: 10 }, box: null }, window);
    expect(view.container.querySelector(".bot-cursor")).toBeNull();

    view.rerender(
      <LiveFrame
        url="data:text/html,<h1>Bakery"
        device="desktop"
        scale={0.5}
        title="home.html"
        writer={null}
      />,
    );
    say({ botloft: "cursor", view: VIEW, point: { x: 10, y: 10 }, box: null }, frame.contentWindow);
    expect(view.container.querySelector(".bot-cursor")).toBeNull();
  });
});

describe("reading a mark", () => {
  test("keeps it inside the page and drops what is not one", () => {
    expect(readMark("hi", PAGE)).toBeUndefined();
    expect(readMark({ botloft: "other" }, PAGE)).toBeUndefined();
    // Without the size it measured in, it cannot be placed.
    expect(readMark({ botloft: "cursor", point: { x: 1, y: 1 }, box: null }, PAGE)).toBeUndefined();
    // Measured in a view zoomed wider than the page: scaled back.
    expect(
      readMark(
        {
          botloft: "cursor",
          view: { width: 1600, height: 1000 },
          point: { x: 800, y: 500 },
          box: null,
        },
        PAGE,
      )?.point,
    ).toEqual({ x: 640, y: 400 });
    expect(
      readMark({ botloft: "cursor", view: VIEW, point: { x: -5, y: 9000 }, box: null }, PAGE)
        ?.point,
    ).toEqual({ x: 0, y: 800 });
    expect(
      readMark(
        {
          botloft: "cursor",
          view: VIEW,
          point: null,
          box: { x: 1200, y: 10, width: 500, height: 20 },
        },
        PAGE,
      )?.box,
    ).toEqual({ x: 1200, y: 10, width: 80, height: 20 });
    expect(
      readMark(
        {
          botloft: "cursor",
          view: VIEW,
          point: { x: "1", y: 2 },
          box: { x: 2000, y: 0, width: 5, height: 5 },
        },
        PAGE,
      ),
    ).toEqual({ point: null, box: null });
  });
});
