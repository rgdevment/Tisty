import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { type PageMenuActs, type PageMenuAsked, pageChoices, pasteHeard } from "../ui/pageMenu";

const asking = (more: Partial<PageMenuAsked> = {}): PageMenuAsked => ({
  words: null,
  cuttable: true,
  said: null,
  spelled: null,
  formattable: true,
  limited: false,
  block: false,
  pasting: false,
  pasteByKeys: false,
  ...more,
});

const acting = (): PageMenuActs => ({
  guessed: vi.fn(),
  learned: vi.fn(),
  cut: vi.fn(),
  copy: vi.fn(),
  paste: vi.fn(),
  format: vi.fn(),
  insert: vi.fn(),
  blockActed: vi.fn(),
});

const keysOf = (asked: PageMenuAsked) => pageChoices(asked, acting()).map((one) => one.key);

const pick = (key: string, asked: PageMenuAsked, acts: PageMenuActs) =>
  pageChoices(asked, acts)
    .find((one) => one.key === key)
    ?.onPick?.();

const BLOCKS = ["insert", "block-up", "block-down", "block-double", "block-drop"];

describe("the menu of a document page", () => {
  it("offers nothing when nothing is chosen, pasted or blocked", () => {
    expect(keysOf(asking())).toEqual([]);
  });

  it("offers the guesses and then learning when the word is wrong", () => {
    const spelled = { wrong: true, guesses: ["prueba", "prueve"] };
    const rows = pageChoices(asking({ said: "prueva", spelled }), acting());
    expect(rows.map((one) => one.key)).toEqual(["guess-prueba", "guess-prueve", "learn"]);
    expect(rows[2].label).toContain("prueva");
    expect(rows.every((one) => !one.apart)).toBe(true);
  });

  it("shows a disabled row when a wrong word has no guesses", () => {
    const spelled = { wrong: true, guesses: [] };
    const rows = pageChoices(asking({ said: "xyzq", spelled }), acting());
    expect(rows.map((one) => one.key)).toEqual(["guess-none", "learn"]);
    expect(rows[0]).toMatchObject({ label: t("spellNone"), off: true });
  });

  it("offers no spelling for a right word, an unknown spelling or no word", () => {
    const right = { wrong: false, guesses: ["x"] };
    const wrong = { wrong: true, guesses: ["x"] };
    expect(keysOf(asking({ said: "casa", spelled: right }))).toEqual([]);
    expect(keysOf(asking({ said: "casa", spelled: null }))).toEqual([]);
    expect(keysOf(asking({ said: null, spelled: wrong }))).toEqual([]);
  });

  it("offers cut and copy only with words, and paste only when pasting", () => {
    expect(keysOf(asking({ words: "uno", formattable: false }))).toEqual(["cut", "copy"]);
    expect(keysOf(asking({ pasting: true }))).toEqual(["paste", "paste-plain"]);
    const both = pageChoices(asking({ words: "uno", formattable: false, pasting: true }), acting());
    expect(both.map((one) => one.label)).toEqual([
      t("cutIt"),
      t("copyIt"),
      t("pasteIt"),
      t("pastePlain"),
    ]);
  });

  it("offers every format with words that can be formatted", () => {
    const keys = keysOf(asking({ words: "uno" }));
    expect(keys.slice(0, 2)).toEqual(["cut", "copy"]);
    expect(keys.filter((key) => key.startsWith("format-")).length).toBeGreaterThan(3);
  });

  it("offers no formats when the words cannot be formatted", () => {
    const keys = keysOf(asking({ words: "uno", formattable: false }));
    expect(keys.some((key) => key.startsWith("format-"))).toBe(false);
  });

  it("offers no formats and no blocks in an Agents file", () => {
    const keys = keysOf(asking({ words: "uno", limited: true, pasting: true }));
    expect(keys).toEqual(["cut", "copy", "paste", "paste-plain"]);
    expect(keysOf(asking({ block: true, limited: true }))).toEqual([]);
  });

  it("offers the block acts only with no words chosen", () => {
    expect(keysOf(asking({ block: true }))).toEqual(BLOCKS);
    expect(keysOf(asking({ block: true, words: "uno", formattable: false }))).toEqual([
      "cut",
      "copy",
    ]);
  });

  it("offers no block acts when no block was under the point", () => {
    expect(keysOf(asking({ block: false }))).toEqual([]);
  });

  it("marks the drop as dangerous and apart", () => {
    const rows = pageChoices(asking({ block: true }), acting());
    expect(rows.find((one) => one.key === "block-drop")).toMatchObject({
      danger: true,
      apart: true,
    });
  });

  it("sets the first row of each later group apart and no other first row", () => {
    const spelled = { wrong: true, guesses: ["prueba"] };
    const asked = asking({ said: "prueva", spelled, words: "prueva", pasting: true });
    const rows = pageChoices(asked, acting());
    expect(rows.filter((one) => one.apart).map((one) => one.key)).toEqual(["cut", "format-bold"]);
    expect(rows[0].apart).toBe(false);
  });

  it("does not set the first group apart whichever group it is", () => {
    const rows = pageChoices(asking({ pasting: true, block: true }), acting());
    expect(rows.map((one) => [one.key, Boolean(one.apart)])).toEqual([
      ["paste", false],
      ["paste-plain", false],
      ["insert", true],
      ["block-up", false],
      ["block-down", false],
      ["block-double", false],
      ["block-drop", true],
    ]);
  });

  it("calls the matching act with its arguments when a row is picked", () => {
    const spelled = { wrong: true, guesses: ["prueba"] };
    const asked = asking({ said: "prueva", spelled, words: "prueva", pasting: true });
    const acts = acting();
    for (const key of ["guess-prueba", "learn", "cut", "copy", "paste", "paste-plain"])
      pick(key, asked, acts);
    pick("format-bold", asked, acts);
    expect(acts.guessed).toHaveBeenCalledWith("prueba");
    expect(acts.learned).toHaveBeenCalledTimes(1);
    expect(acts.cut).toHaveBeenCalledTimes(1);
    expect(acts.copy).toHaveBeenCalledTimes(1);
    expect(acts.paste).toHaveBeenNthCalledWith(1, false);
    expect(acts.paste).toHaveBeenNthCalledWith(2, true);
    expect(acts.format).toHaveBeenCalledWith("**", "**");
  });

  it("calls insert and the block acts with their names", () => {
    const asked = asking({ block: true });
    const acts = acting();
    for (const key of BLOCKS) pick(key, asked, acts);
    expect(acts.insert).toHaveBeenCalledTimes(1);
    expect(vi.mocked(acts.blockActed).mock.calls).toEqual([["up"], ["down"], ["double"], ["drop"]]);
  });
});

describe("a choice that cannot be taken out", () => {
  it("offers copying it, never cutting it", () => {
    const keys = pageChoices(asking({ words: "| a |", cuttable: false }), acting()).map(
      (one) => one.key,
    );
    expect(keys).toContain("copy");
    expect(keys).not.toContain("cut");
  });
});

describe("a paste the window cannot make for the page", () => {
  it("is shown off, with the keys that paste instead", () => {
    const acts = acting();
    const rows = pageChoices(asking({ pasting: true, pasteByKeys: true }), acts);
    expect(rows.map((one) => [one.key, one.off])).toEqual([
      ["paste", true],
      ["paste-plain", true],
    ]);
    expect(rows[0].hint).toContain("V");
    expect(rows[0].onPick).toBeUndefined();
  });
});

describe("a paste asked of the window", () => {
  it("is heard when the page's own paste comes", async () => {
    document.body.innerHTML = '<article class="page" contenteditable="true"></article>';
    const sheet = document.querySelector(".page") as Element;
    const heard = pasteHeard(sheet, Promise.resolve(true));
    sheet.dispatchEvent(new Event("paste", { bubbles: true }));
    await expect(heard).resolves.toBe(true);
  });

  it("is not heard from a paste that lands beside the sheet", async () => {
    vi.useFakeTimers();
    try {
      document.body.innerHTML = '<article class="page" contenteditable="true"></article>';
      const heard = pasteHeard(document.querySelector(".page") as Element, Promise.resolve(true));
      document.body.dispatchEvent(new Event("paste", { bubbles: true }));
      await vi.advanceTimersByTimeAsync(2000);
      await expect(heard).resolves.toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });

  it("is told as not heard when the window could not send it", async () => {
    await expect(pasteHeard(document.body, Promise.resolve(false))).resolves.toBe(false);
  });

  it("is told as not heard when no paste comes in time", async () => {
    vi.useFakeTimers();
    try {
      const heard = pasteHeard(document.body, Promise.resolve(true));
      await vi.advanceTimersByTimeAsync(2000);
      await expect(heard).resolves.toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });
});
