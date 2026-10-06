import { describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { es } from "../../i18n/es";
import { ptBR } from "../../i18n/pt-BR";
import { FAKE_CATALOG } from "../../lib/fakeCatalog";

// The daemon's sheets, one file each (spec 26.2).
const sheets = import.meta.glob("../../../../crates/botloftd/catalog/*.toml", {
  query: "?raw",
  import: "default",
  eager: true,
});

const ids = Object.keys(sheets).map((path) => path.replace(/^.*\/(.+)\.toml$/, "$1"));

describe("the roles the owner reads about", () => {
  test("there is a sheet for each, and the fake daemon lists the same", () => {
    expect(ids.length).toBeGreaterThanOrEqual(26);
    expect(FAKE_CATALOG.map((role) => role.id).sort()).toEqual([...ids].sort());
  });

  test.each([
    ["English", en],
    ["Portuguese", ptBR],
    ["Spanish", es],
  ])("every sheet has its text in %s", (_name, messages) => {
    const roles = messages.catalog.roles as Record<
      string,
      { name: string; role: string; summary: string; about: string; when: string; pairs: string }
    >;
    expect(Object.keys(roles).sort()).toEqual([...ids].sort());
    for (const id of ids) {
      const text = roles[id];
      for (const field of ["name", "role", "summary", "about", "when", "pairs"] as const) {
        expect(text?.[field]?.trim(), `${id}.${field}`).toBeTruthy();
      }
      // The daemon takes a role of one line, up to 200 characters.
      expect(text?.role.length, `${id}.role`).toBeLessThanOrEqual(200);
      expect(text?.role.includes("\n")).toBe(false);
    }
  });
});
