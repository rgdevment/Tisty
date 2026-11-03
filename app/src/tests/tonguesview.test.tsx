import { fireEvent, render, screen } from "@testing-library/react";
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

  it("stands under no heading when no sheet is drawn", () => {
    expect(standingHead()).toBe(-1);
  });
});
