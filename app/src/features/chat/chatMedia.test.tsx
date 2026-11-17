import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { BotFile } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";

const MINUTE = 60_000;

beforeEach(() => {
  // jsdom plays nothing: playing and pausing only tell the page they did.
  const state = (element: HTMLMediaElement, paused: boolean, event: string) => {
    Object.defineProperty(element, "paused", { configurable: true, value: paused });
    element.dispatchEvent(new Event(event));
  };
  vi.spyOn(HTMLMediaElement.prototype, "play").mockImplementation(function (
    this: HTMLMediaElement,
  ) {
    state(this, false, "play");
    return Promise.resolve();
  });
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(function (
    this: HTMLMediaElement,
  ) {
    state(this, true, "pause");
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

/** A crew "Ops" with @scout, idle. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, scout };
}

function share(fake: FakeBotloft, botId: string, files: BotFile[]) {
  const call = fake.chat.tool(botId, "mcp__botloft__share_file", {
    summary: files.map((file) => file.name).join(", "),
    input: JSON.stringify({ files: files.map((file) => file.path) }),
  });
  fake.chat.finish(call, JSON.stringify({ shown: files }));
}

async function openScout(fake: FakeBotloft) {
  const rendered = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return rendered;
}

const cards = () => screen.findByRole("list", { name: "Files shared by Scout" });

describe("sounds, videos and pictures in the chat", () => {
  test("a shared sound plays, pauses and jumps in its card", async () => {
    const { fake, scout } = crew();
    const voice = fake.files.add(scout.id, "voice.mp3", { text: "ID3" });
    share(fake, scout.id, [voice]);
    await openScout(fake);

    const card = within(await cards());
    const play = await card.findByRole("button", { name: "Play voice.mp3" });
    const audio = document.querySelector("audio") as HTMLAudioElement;
    expect(audio.getAttribute("src")).toBe("http://127.0.0.1:45710/media/voice.mp3");
    expect(fake.calls.some((call) => call.method === "files.media")).toBe(true);

    Object.defineProperty(audio, "duration", { configurable: true, value: 65 });
    fireEvent.loadedMetadata(audio);
    expect(card.getByText("0:00 / 1:05")).toBeDefined();

    fireEvent.click(play);
    const pause = await card.findByRole("button", { name: "Pause voice.mp3" });
    fireEvent.click(pause);
    expect(await card.findByRole("button", { name: "Play voice.mp3" })).toBeDefined();

    fireEvent.change(card.getByRole("slider", { name: "Where voice.mp3 is playing" }), {
      target: { value: "30" },
    });
    expect(audio.currentTime).toBe(30);
    expect(card.getByText("0:30 / 1:05")).toBeDefined();
  });

  test("starting one sound pauses the one playing", async () => {
    const { fake, scout } = crew();
    const one = fake.files.add(scout.id, "one.mp3", { text: "1" });
    const two = fake.files.add(scout.id, "two.wav", { text: "2" });
    share(fake, scout.id, [one, two]);
    await openScout(fake);

    const card = within(await cards());
    fireEvent.click(await card.findByRole("button", { name: "Play one.mp3" }));
    await card.findByRole("button", { name: "Pause one.mp3" });
    fireEvent.click(card.getByRole("button", { name: "Play two.wav" }));
    expect(await card.findByRole("button", { name: "Play one.mp3" })).toBeDefined();
    expect(card.getByRole("button", { name: "Pause two.wav" })).toBeDefined();
  });

  test("a shared video plays in its card, and in the files panel", async () => {
    const { fake, scout } = crew();
    const clip = fake.files.add(scout.id, "teaser.mp4", { text: "ftyp" });
    share(fake, scout.id, [clip]);
    await openScout(fake);

    const list = await cards();
    const video = await waitFor(() => {
      const found = list.querySelector("video");
      expect(found).not.toBeNull();
      return found as HTMLVideoElement;
    });
    expect(video.getAttribute("src")).toBe("http://127.0.0.1:45710/media/teaser.mp4");
    expect(video.getAttribute("aria-label")).toBe("teaser.mp4");
    expect(video.controls).toBe(true);

    fireEvent.click(within(list).getByRole("button", { name: /^Preview/ }));
    const panel = await screen.findByRole("complementary", { name: "Files from Scout" });
    await waitFor(() => expect(panel.querySelector("video")).not.toBeNull());
    expect(within(panel).queryByText(/no preview/)).toBeNull();
  });

  test("a sound that cannot play says so, after asking for a new link once", async () => {
    const { fake, scout } = crew();
    const odd = fake.files.add(scout.id, "odd.m4a", { text: "?" });
    share(fake, scout.id, [odd]);
    await openScout(fake);

    const card = within(await cards());
    await card.findByRole("button", { name: "Play odd.m4a" });
    fireEvent.error(document.querySelector("audio") as HTMLAudioElement);
    await card.findByRole("button", { name: "Play odd.m4a" });
    fireEvent.error(document.querySelector("audio") as HTMLAudioElement);
    expect(await card.findByRole("alert")).toHaveProperty(
      "textContent",
      "This can't play here. Open it in your own program.",
    );
    const asked = fake.calls.filter((call) => call.method === "files.media");
    expect(asked).toHaveLength(2);
    // Opening it in the owner's own program is still there.
    expect(card.getByRole("button", { name: "Open" })).toBeDefined();
  });

  test("a picture opens as big as the window and closes with Escape", async () => {
    const { fake, scout } = crew();
    const cover = fake.files.add(scout.id, "cover.png", { text: "png" });
    share(fake, scout.id, [cover]);
    await openScout(fake);

    const card = within(await cards());
    fireEvent.click(await card.findByRole("button", { name: "See cover.png bigger" }));
    const viewer = await screen.findByRole("dialog", { name: "Picture: cover.png" });
    expect(within(viewer).getByRole("img", { name: "cover.png" })).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());

    fireEvent.click(card.getByRole("button", { name: "See cover.png bigger" }));
    const again = await screen.findByRole("dialog", { name: "Picture: cover.png" });
    fireEvent.click(again);
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });
});

describe("what the bot made in a turn", () => {
  /** Scout runs a command that makes files, maybe shares some, and ends. */
  function turn(fake: FakeBotloft, botId: string, make: () => BotFile[], shared: number[] = []) {
    void fake.call("messages.send", { botId, body: "Make the teaser" });
    const command = fake.chat.tool(botId, "Bash", { summary: "ffmpeg -i in.mov teaser.mp4" });
    fake.chat.finish(command, "done");
    const made = make();
    const toShare = shared.map((index) => made[index] as BotFile);
    if (toShare.length > 0) {
      share(fake, botId, toShare);
    }
    fake.chat.reply(botId, "The teaser is ready.");
    fake.chat.turn(botId);
    return made;
  }

  test("its pictures, sounds and videos show under the turn, but not what it shared", async () => {
    const { fake, scout } = crew();
    fake.files.add(scout.id, "old.png", { text: "old", at: fake.now - 30 * MINUTE });
    turn(
      fake,
      scout.id,
      () => [
        fake.files.add(scout.id, "teaser.mp4", {
          text: "ftyp",
          folder: "out",
          at: fake.now - 2000,
        }),
        fake.files.add(scout.id, "cover.png", { text: "png", folder: "out" }),
        fake.files.add(scout.id, "notes.md", { text: "# notes" }),
        fake.files.add(scout.id, "poster.png", { text: "png" }),
      ],
      [3],
    );
    await openScout(fake);

    const made = await screen.findByRole("region", {
      name: "Pictures, sounds and videos Scout made",
    });
    expect(within(made).getByRole("heading", { name: "Also made" })).toBeDefined();
    const items = within(made).getAllByRole("listitem");
    const names = items.map((item) => item.querySelector("[title^='C:']")?.textContent);
    expect(names).toEqual(["teaser.mp4", "cover.png"]);
    await waitFor(() => expect(made.querySelector("video")).not.toBeNull());
    // The shared one has its own card, once.
    expect(within(await cards()).getByText("poster.png")).toBeDefined();
    expect(within(made).queryByText("poster.png")).toBeNull();
  });

  test("past six, the rest is a link to the files panel", async () => {
    const { fake, scout } = crew();
    turn(fake, scout.id, () =>
      Array.from({ length: 8 }, (_, index) =>
        fake.files.add(scout.id, `frame-${index}.png`, { text: "png", folder: "frames" }),
      ),
    );
    await openScout(fake);

    const made = await screen.findByRole("region", {
      name: "Pictures, sounds and videos Scout made",
    });
    expect(within(made).getAllByRole("listitem")).toHaveLength(6);
    fireEvent.click(within(made).getByRole("button", { name: "2 more in Files" }));
    const panel = await screen.findByRole("complementary", { name: "Files from Scout" });
    expect(await within(panel).findByText("frame-7.png")).toBeDefined();
  });

  test("a turn that made no media shows nothing more", async () => {
    const { fake, scout } = crew();
    turn(fake, scout.id, () => [fake.files.add(scout.id, "notes.md", { text: "# notes" })]);
    await openScout(fake);

    await screen.findAllByText("The teaser is ready.");
    expect(screen.queryByRole("region", { name: /Scout made/ })).toBeNull();
  });
});
