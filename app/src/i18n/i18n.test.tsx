import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../App";
import type { Client } from "../lib/client";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { when } from "../lib/format";
import { currentLocale, LOCALES, type Messages, setLocaleChoice, systemLocale, t } from ".";
import { en } from "./en";
import { es } from "./es";
import { ptBR } from "./pt-BR";

afterEach(() => {
  cleanup();
  setLocaleChoice("system");
});

/** Every leaf of a messages tree, with its path, to compare languages. */
function leaves(tree: object, path = ""): [string, unknown][] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === "object" && value !== null
      ? leaves(value, `${path}${key}.`)
      : [[`${path}${key}`, value] as [string, unknown]],
  );
}

describe("languages", () => {
  test("the system language picks the first supported one, English otherwise", () => {
    expect(systemLocale(["pt-BR", "en-US"])).toBe("pt-BR");
    expect(systemLocale(["pt-PT"])).toBe("pt-BR");
    expect(systemLocale(["es-419", "en"])).toBe("es");
    expect(systemLocale(["de-DE", "es-ES"])).toBe("es");
    expect(systemLocale(["fr-FR"])).toBe("en");
    expect(systemLocale([])).toBe("en");
  });

  test("every language has every text, and none is left empty", () => {
    const reference = leaves(en)
      .map(([path]) => path)
      .sort();
    for (const messages of [ptBR, es] as Messages[]) {
      const found = leaves(messages);
      expect(found.map(([path]) => path).sort()).toEqual(reference);
      for (const [path, value] of found) {
        expect(
          typeof value === "function" || (typeof value === "string" && value.length > 0),
          path,
        ).toBe(true);
      }
    }
    expect(LOCALES.map((locale) => locale.id)).toEqual(["en", "pt-BR", "es"]);
  });

  test("the choice switches the app at once and is remembered", async () => {
    const fake = new FakeBotloft();
    render(<App host={new FakeHost()} connect={() => fake as Client} />);
    expect(await screen.findByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Language" }));
    fireEvent.click(screen.getByRole("menuitemradio", { name: /Português/ }));
    expect(screen.getByRole("heading", { name: "Boas-vindas ao Botloft" })).toBeDefined();
    expect(document.documentElement.lang).toBe("pt-BR");
    expect(localStorage.getItem("botloft.locale")).toBe("pt-BR");

    fireEvent.click(screen.getByRole("button", { name: "Idioma" }));
    expect(
      screen.getByRole("menuitemradio", { name: /Português/ }).getAttribute("aria-checked"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("menuitemradio", { name: "Español" }));
    expect(screen.getByRole("heading", { name: "Te damos la bienvenida a Botloft" })).toBeDefined();

    act(() => setLocaleChoice("system"));
    expect(screen.getByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();
  });

  test("text outside React and dates follow the language", () => {
    setLocaleChoice("es");
    expect(currentLocale()).toBe("es");
    expect(t().common.tryAgain).toBe("Reintentar");
    const lastYear = new Date(new Date().getFullYear() - 1, 0, 15, 9, 30).getTime();
    expect(when(lastYear)).toMatch(/ene/i);
    setLocaleChoice("en");
    expect(when(lastYear)).toMatch(/Jan/);
  });
});
