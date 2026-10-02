import { describe, expect, test } from "vitest";
import { fileKind } from "./kinds";

const OCTET = "application/octet-stream";

describe("fileKind", () => {
  test("office files by their media type", () => {
    expect(
      fileKind(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "report.docx",
      ),
    ).toBe("word");
    expect(
      fileKind("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "q3.xlsx"),
    ).toBe("sheet");
    expect(fileKind("text/csv", "plan.csv")).toBe("sheet");
    expect(fileKind("application/pdf", "contract.pdf")).toBe("pdf");
  });

  test("old and open formats by extension, though they come as octet-stream", () => {
    expect(fileKind(OCTET, "Old Letter.DOC")).toBe("word");
    expect(fileKind(OCTET, "budget.ods")).toBe("sheet");
    expect(fileKind(OCTET, "pitch.ppt")).toBe("slides");
    expect(fileKind(OCTET, "backup.7z")).toBe("archive");
    expect(fileKind(OCTET, "call.m4a")).toBe("audio");
  });

  test("media, code, text and the rest", () => {
    expect(fileKind("image/png", "chart.png")).toBe("image");
    expect(fileKind("audio/mpeg", "memo.mp3")).toBe("audio");
    expect(fileKind("video/mp4", "demo.mp4")).toBe("video");
    expect(fileKind("text/plain", "main.rs")).toBe("code");
    expect(fileKind("application/json", "data.json")).toBe("code");
    expect(fileKind("text/markdown", "notes.md")).toBe("text");
    expect(fileKind(OCTET, "blob.bin")).toBe("other");
    expect(fileKind(OCTET, ".docx")).toBe("other");
  });
});
