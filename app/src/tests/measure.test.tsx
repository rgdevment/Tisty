import { readFileSync } from "node:fs";
import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { MEASURES, measureFor, measureOf, SHOWN_MEASURE, useMeasure } from "../breadth";
import type { Filed } from "../core";
import { t } from "../locales";
import { MEASURE_WORDS, measureChoice, widens } from "../measureMenu";
import type { Place } from "../places";
import { LivelyBelow } from "../ui/liveSheet";
import { inTheRepo } from "./repo";

const KEY = "tisty.measure";
const saved = () => JSON.parse(localStorage.getItem(KEY) ?? "{}");
const filed = (id: string) => ({ id }) as unknown as Filed;
const now = (id: string) => {
  // measureChoice reads the live store, so its marked option is the current one
  const on = measureChoice(filed(id)).into?.choices.find((c) => c.on);
  return on?.key;
};

beforeEach(() => {
  for (const id of ["d1", "d2", "d3"]) measureFor(id, "normal");
  localStorage.clear();
});

describe("the width of a document's text", () => {
  it("is normal until someone chooses", () => {
    expect(SHOWN_MEASURE).toBe("normal");
    expect(now("d1")).toBe("normal");
    expect(measureOf("d1", {})).toBe("normal");
  });

  it("keeps what is chosen and reads it back", () => {
    measureFor("d1", "wide");
    expect(saved()).toEqual({ d1: "wide" });
    expect(now("d1")).toBe("wide");
    expect(measureOf("d1", saved())).toBe("wide");
  });

  it("forgets the entry when normal is chosen again", () => {
    measureFor("d1", "full");
    measureFor("d1", "normal");
    expect(saved()).toEqual({});
  });

  it("drops invalid values and garbage when read from the computer", () => {
    render(<Shown id="d1" />);
    const readAgain = (raw: string) => {
      localStorage.setItem(KEY, raw);
      act(() => {
        window.dispatchEvent(new StorageEvent("storage", { key: KEY }));
      });
    };
    readAgain(JSON.stringify({ d1: "wide", d2: "huge", d3: 7 }));
    expect(now("d1")).toBe("wide");
    expect(now("d2")).toBe("normal");
    expect(now("d3")).toBe("normal");
    readAgain("{not json");
    expect(now("d1")).toBe("normal");
    readAgain("[1,2]");
    expect(now("d1")).toBe("normal");
  });

  it("keeps each document's own", () => {
    measureFor("d1", "narrow");
    measureFor("d2", "full");
    expect(saved()).toEqual({ d1: "narrow", d2: "full" });
    expect(now("d1")).toBe("narrow");
    expect(now("d2")).toBe("full");
    expect(now("d3")).toBe("normal");
    measureFor("d1", "normal");
    expect(saved()).toEqual({ d2: "full" });
  });
});

function Shown({ id }: { id?: string }) {
  return <output>{useMeasure(id)}</output>;
}

describe("useMeasure", () => {
  it("draws again when the preference changes", () => {
    render(<Shown id="d1" />);
    expect(screen.getByRole("status").textContent).toBe("normal");
    act(() => measureFor("d1", "wide"));
    expect(screen.getByRole("status").textContent).toBe("wide");
  });

  it("answers normal with no document", () => {
    measureFor("d1", "wide");
    render(<Shown id={undefined} />);
    expect(screen.getByRole("status").textContent).toBe("normal");
  });

  it("follows the document when the id changes, including to none", () => {
    measureFor("d1", "narrow");
    const { rerender } = render(<Shown id="d1" />);
    expect(screen.getByRole("status").textContent).toBe("narrow");
    rerender(<Shown id="d2" />);
    expect(screen.getByRole("status").textContent).toBe("normal");
    rerender(<Shown id={undefined} />);
    expect(screen.getByRole("status").textContent).toBe("normal");
    rerender(<Shown id="d1" />);
    expect(screen.getByRole("status").textContent).toBe("narrow");
  });
});

describe("the text width choice in the menu", () => {
  it("offers the four in order, the current one on", () => {
    measureFor("d1", "wide");
    const choice = measureChoice(filed("d1"));
    expect(choice.key).toBe("measure");
    const inside = choice.into?.choices ?? [];
    expect(inside.map((c) => c.key)).toEqual(["narrow", "normal", "wide", "full"]);
    expect(inside.map((c) => !!c.on)).toEqual([false, false, true, false]);
  });

  it("keeps the one picked", () => {
    const inside = () => measureChoice(filed("d1")).into?.choices ?? [];
    inside()
      .find((c) => c.key === "full")
      ?.onPick?.();
    expect(saved()).toEqual({ d1: "full" });
    expect(inside().find((c) => c.on)?.key).toBe("full");
  });

  it("is named in english and spanish", () => {
    const labels = [
      "measure",
      "measureNarrow",
      "measureNormal",
      "measureWide",
      "measureFull",
      "measureNeedsNoSheets",
    ] as const;
    const locales = readFileSync("src/locales.ts", "utf8");
    for (const name of labels) {
      expect(locales.split(`\n  ${name}:`)).toHaveLength(3);
      expect(t(name)).not.toBe("");
    }
    expect(MEASURES).toHaveLength(4);
  });
});

describe("what the window's stylesheets say about the measure", () => {
  const index = readFileSync("src/index.css", "utf8");
  const lively = readFileSync(inTheRepo("packages", "tisty-sheet", "src", "lively.css"), "utf8");
  const rule = (css: string, selector: string) => {
    const from = css.indexOf(`${selector} {`);
    return from < 0 ? "" : css.slice(from, css.indexOf("}", from));
  };

  it("sets the three widths and leaves normal to the default", () => {
    expect(rule(index, '[data-measure="narrow"]')).toMatch(/--measure-across:\s*672px/);
    expect(rule(index, '[data-measure="wide"]')).toMatch(/--measure-across:\s*1112px/);
    expect(rule(index, '[data-measure="full"]')).toMatch(/--measure-across:\s*none/);
    expect(index).toMatch(/max-width:\s*var\(--measure-across,\s*872px\)/);
  });

  it("casts no shadow from the page on screen", () => {
    const screens = index.slice(index.indexOf("@media screen"));
    const page = screens.match(/\.lively-room > \.tisty-sheet\s*\{([^}]*)\}/)?.[1] ?? "";
    expect(page).not.toBe("");
    expect(page).not.toMatch(/box-shadow/);
  });

  it("holds what hangs below the page to the same measure as the page", () => {
    expect(index).toMatch(
      /\.lively-room > :is\([^)]*\.tisty-sheet[^)]*\.lively-below[^)]*\)\s*\{[^}]*max-width:\s*var\(--measure-across,\s*872px\)/,
    );
  });

  it("no longer holds the sheet to 68ch", () => {
    expect(lively).not.toContain("68ch");
  });
});

describe("what hangs below the whole-document page", () => {
  it.each([false, true])("wears the class the measure holds when folds is %s", (folds) => {
    const { container } = render(
      <LivelyBelow folds={folds} width="w-full">
        <p>under the page</p>
      </LivelyBelow>,
    );
    const wrapper = container.firstElementChild as HTMLElement;
    expect(wrapper.classList.contains("lively-below")).toBe(true);
    expect(wrapper.className).not.toContain("max-w-[1100px]");
  });
});

describe("where the text width can be chosen", () => {
  it.each([
    ["plain", true],
    ["ideas", false],
    ["diary", false],
    ["agents", false],
  ] as [Place, boolean][])("says %s widens: %s", (place, widened) => {
    expect(widens(place)).toBe(widened);
  });

  it("names every width with a word of its own", () => {
    expect(MEASURES.map((one) => MEASURE_WORDS[one])).toEqual([
      "measureNarrow",
      "measureNormal",
      "measureWide",
      "measureFull",
    ]);
  });
});
