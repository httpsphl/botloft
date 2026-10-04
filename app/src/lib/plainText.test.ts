import { describe, expect, test } from "vitest";
import { plainText } from "./plainText";

describe("plainText", () => {
  test("drops strong, emphasis, code and strikethrough marks", () => {
    expect(plainText("**Vendas do dia, 03/10:** 1 venda nova")).toBe(
      "Vendas do dia, 03/10: 1 venda nova",
    );
    expect(plainText("__done__, *now* and _soon_, `npm test`, ~~old~~")).toBe(
      "done, now and soon, npm test, old",
    );
    expect(plainText("***both***")).toBe("both");
  });

  test("keeps a lone star or underscore inside words", () => {
    expect(plainText("rename snake_case_name, 2*3*4 = 24")).toBe(
      "rename snake_case_name, 2*3*4 = 24",
    );
  });

  test("drops headings, quotes and list marks, joining lines", () => {
    expect(plainText("## Report\n> - first\n2. second\n- [x] third")).toBe(
      "Report first second third",
    );
  });

  test("keeps the text of links and images, and leaves out fences and rules", () => {
    expect(plainText("See [the docs](https://example.com) ![chart](a.png)")).toBe(
      "See the docs chart",
    );
    expect(plainText("Run:\n```sh\nls\n```\n---\nDone")).toBe("Run: ls Done");
  });

  test("table cells become words, escapes their characters", () => {
    expect(plainText("| Item | Qty |\n| --- | --- |\n| Cake | 2 |")).toBe("Item Qty Cake 2");
    expect(plainText("1\\*2 is \\#1")).toBe("1*2 is #1");
  });
});
