import { describe, expect, it, vi } from "vitest";
import type { Choice } from "../ui/Menu";
import { type ColumnWays, columnChoices, type Headed, wordsOf } from "../ui/tableChoices";

const ways = (): ColumnWays => ({
  sort: vi.fn(),
  filter: vi.fn(),
  asking: vi.fn(),
  retype: vi.fn(),
  rename: vi.fn(),
  insert: vi.fn(),
  remove: vi.fn(),
  markdown: vi.fn(),
});

const head = (type: string, total = "", currency = ""): Headed => ({
  name: "Precio",
  type,
  total,
  currency,
});

const inside = (choices: Choice[], key: string): Choice[] =>
  choices.find((one) => one.key === key)?.into?.choices ?? [];

const picked = (choices: Choice[], ...path: string[]) => {
  let now = choices;
  for (const key of path.slice(0, -1)) now = inside(now, key);
  now.find((one) => one.key === path[path.length - 1])?.onPick?.();
};

describe("the words in a head's braces", () => {
  // Type, currency and total, then the words written between the braces.
  const WORDS = `
money CLP sum => money CLP sum
money - sum   => money sum
number - -    => number
text - -      =>
- - -         =>
text - count  => text count
date USD max  => date max
`;

  it("are the type, the currency of money and the total, and none for plain text", () => {
    for (const line of WORDS.trim().split("\n")) {
      const [asked, said = ""] = line.split("=>").map((one) => one.trim());
      const [type, currency, total] = asked.split(/\s+/).map((one) => (one === "-" ? "" : one));
      expect(wordsOf(type, currency, total), line).toBe(said);
    }
  });
});

describe("a column's menu", () => {
  // A type, then what its menu sorts by and how it filters.
  const KINDS = `
          => Sort A → Z | Sort Z → A | has
number    => Smallest first | Largest first |
date      => Earliest first | Latest first | soon past
check     => No first | Yes first | yes no
select    => Sort A → Z | Sort Z → A | is-música is-video
url       => Sort A → Z | Sort Z → A | has
`;

  it("sorts and filters as its type reads", () => {
    for (const line of KINDS.trim().split("\n")) {
      const [type, said] = line.split("=>").map((one) => one.trim());
      const [up, down, filters] = said.split("|").map((one) => one.trim());
      const choices = columnChoices(head(type), ["Música", "Video"], ways());
      expect(choices[0].label, line).toBe(up);
      expect(choices[1].label, line).toBe(down);
      const filter = choices.find((one) => one.key === "filter");
      const kinds = inside(choices, "filter").map((one) => one.key.replace(/-$/, ""));
      expect(kinds.join(" "), line).toBe(filters);
      expect(filter?.off, line).toBe(filters === "");
    }
  });

  it("offers only the totals that add up for its type, the one in use ticked", () => {
    const totals = (type: string, total = "") =>
      inside(columnChoices(head(type, total), [], ways()), "total").map(
        (one) => `${one.key}${one.icon ? "✓" : ""}`,
      );
    expect(totals("money", "sum")).toEqual(["sum✓", "avg", "min", "max", "count", "none"]);
    expect(totals("date")).toEqual(["min", "max", "count", "none✓"]);
    expect(totals("check")).toEqual(["count", "none✓"]);
  });

  it("writes the type back with each change, keeping what still fits", () => {
    const asked = ways();
    const choices = columnChoices(head("money", "sum", "CLP"), [], asked);
    picked(choices, "total", "avg");
    picked(choices, "currency", "USD");
    picked(choices, "currency", "none");
    picked(choices, "type", "date");
    picked(choices, "type", "check");
    picked(choices, "type", "text");
    expect(vi.mocked(asked.retype).mock.calls.map(([words]) => words)).toEqual([
      "money CLP avg",
      "money USD sum",
      "money sum",
      "date",
      "check",
      "",
    ]);
    expect(columnChoices(head("number"), [], ways()).some((one) => one.key === "currency")).toBe(
      false,
    );
  });

  it("filters by an option with the column's name on its label", () => {
    const asked = ways();
    picked(columnChoices(head("select"), ["Música"], asked), "filter", "is-música");
    expect(asked.filter).toHaveBeenCalledWith({
      kind: "is",
      said: "música",
      label: "Precio: Música",
    });
    const text = ways();
    picked(columnChoices(head(""), [], text), "filter", "has");
    expect(text.asking).toHaveBeenCalled();
  });

  it("ends with what every column has", () => {
    const asked = ways();
    const choices = columnChoices(head("number"), [], asked);
    expect(choices.slice(-6).map((one) => one.key)).toEqual([
      "type",
      "rename",
      "left",
      "right",
      "markdown",
      "remove",
    ]);
    for (const key of ["up", "down", "rename", "left", "right", "markdown", "remove"])
      picked(choices, key);
    expect(asked.sort).toHaveBeenNthCalledWith(1, false);
    expect(asked.sort).toHaveBeenNthCalledWith(2, true);
    expect(asked.insert).toHaveBeenNthCalledWith(1, false);
    expect(asked.insert).toHaveBeenNthCalledWith(2, true);
    expect(asked.rename).toHaveBeenCalled();
    expect(asked.markdown).toHaveBeenCalled();
    expect(asked.remove).toHaveBeenCalled();
  });
});
