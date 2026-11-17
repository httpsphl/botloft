import { describe, expect, test } from "vitest";
import { clock, mediaKind } from "./media";

describe("mediaKind", () => {
  test("by the type, or by the name when the type says nothing", () => {
    expect(mediaKind({ mediaType: "audio/mpeg", name: "a.mp3" })).toBe("audio");
    expect(mediaKind({ mediaType: "video/quicktime", name: "take.mov" })).toBe("video");
    // A card stored before the daemon knew the extension.
    expect(mediaKind({ mediaType: "application/octet-stream", name: "Voice.M4A" })).toBe("audio");
    expect(mediaKind({ mediaType: "application/octet-stream", name: "clip.webm" })).toBe("video");
    expect(mediaKind({ mediaType: "image/png", name: "cover.png" })).toBeNull();
    expect(mediaKind({ mediaType: "application/octet-stream", name: "mp4" })).toBeNull();
  });
});

describe("clock", () => {
  test("minutes and seconds, hours when there are", () => {
    expect(clock(0)).toBe("0:00");
    expect(clock(65.9)).toBe("1:05");
    expect(clock(3723)).toBe("1:02:03");
    expect(clock(Number.NaN)).toBe("0:00");
    expect(clock(Number.POSITIVE_INFINITY)).toBe("0:00");
  });
});
