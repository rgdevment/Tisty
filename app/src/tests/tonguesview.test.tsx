import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Tongues, { standingHead } from "../ui/Tongues";

const doc = (id: string, lang: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title: id,
  folder: null,
  archived: false,
  away: false,
  lang,
  ...more,
});

describe("the switch between languages", () => {
  it("names each language in its own words and opens another, not the one shown", () => {
    const onOpen = vi.fn();
    render(
      <Tongues
        tongues={[doc("es", "es"), doc("en", "en"), doc("pt", "pt", { away: true })]}
        here="es"
        onOpen={onOpen}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Español" }));
    expect(onOpen).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "English" }));
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "en" }));
    expect((screen.getByRole("button", { name: "Português" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    expect(screen.getByRole("button", { name: "Español" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
  });

  it("folds four or more languages into one button that lists them", () => {
    const onOpen = vi.fn();
    render(
      <Tongues
        tongues={[
          doc("es", "es"),
          doc("en", "en"),
          doc("pt", "pt"),
          doc("fr", "fr"),
          doc("de", "de", { away: true }),
        ]}
        here="es"
        onOpen={onOpen}
      />,
    );

    expect(screen.queryByRole("button", { name: "English" })).toBeNull();
    const folded = screen.getByRole("button", { name: /Español/ });
    expect(folded.textContent).toContain("4");

    fireEvent.click(folded);
    const menu = screen.getByRole("menu");
    expect(screen.queryByRole("menuitem", { name: /Deutsch/ })).toBeNull();
    fireEvent.click(within(menu).getByRole("menuitem", { name: /Français/ }));
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "fr" }));
  });

  it("keeps three living languages side by side, an archived one beside them", () => {
    render(
      <Tongues
        tongues={[
          doc("es", "es"),
          doc("en", "en"),
          doc("pt", "pt"),
          doc("de", "de", { away: true }),
        ]}
        here="es"
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "English" })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Deutsch" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("lets nine or more languages be found by name or code in the list", () => {
    const langs = ["es", "en", "pt", "fr", "de", "it", "nl", "ru", "ja"];
    render(<Tongues tongues={langs.map((lang) => doc(lang, lang))} here="es" onOpen={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: /Español/ }));
    const find = screen.getByRole("searchbox", { name: "Find a language" });
    fireEvent.change(find, { target: { value: "franc" } });
    expect(screen.getAllByRole("menuitem").map((one) => one.textContent)).toEqual(["FRFrançais"]);
    fireEvent.change(find, { target: { value: "ja" } });
    expect(screen.getAllByRole("menuitem").map((one) => one.textContent)).toEqual(["JA日本語"]);
  });

  it("stands under no heading when no sheet is drawn", () => {
    expect(standingHead()).toBe(-1);
  });
});
