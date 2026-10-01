import { describe, expect, test } from "vitest";
import { systemName } from "./system";

describe("systemName", () => {
  test("reads the system from the app window's user agent", () => {
    expect(
      systemName(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36 Edg/140.0",
      ),
    ).toBe("Windows");
    expect(
      systemName(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)",
      ),
    ).toBe("macOS");
    expect(
      systemName(
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)",
      ),
    ).toBe("Linux");
  });

  test("anything unknown is Windows", () => {
    expect(systemName("")).toBe("Windows");
    expect(systemName("Mozilla/5.0 (Linux; Android 14)")).toBe("Windows");
  });
});
