import { describe, expect, it } from "vitest";
import { shelved } from "../archive";
import type { List, Task } from "../core";

const lists: List[] = [
  { id: "01W", name: "Work", order: "a0" },
  { id: "01H", name: "Home", order: "a1" },
];

const closed = (id: string, some: Partial<Task>): Task =>
  ({
    id,
    title: id,
    status: "done",
    priority: "unset",
    order: "a0",
    ...some,
  }) as Task;

const bands = (rows: { band: string }[]) => rows.map((row) => row.band);

describe("shelving the archive by an axis", () => {
  it("puts a task under every topic it carries, because an index is not an inbox", () => {
    const rows = shelved(
      [closed("01", { tags: ["release", "windows"] }), closed("02", { tags: ["release"] })],
      "tag",
      lists,
    );

    expect(bands(rows).filter((band) => band === "#release")).toHaveLength(2);
    expect(bands(rows)).toContain("#windows");
  });

  it("keeps what carries nothing, in a band of its own at the end", () => {
    const rows = shelved([closed("01", { tags: ["release"] }), closed("02", {})], "tag", lists);

    const seen = bands(rows);
    expect(seen[seen.length - 1]).toBe("Untagged");
  });

  it("follows the order the lists were given, not the alphabet", () => {
    const rows = shelved(
      [closed("01", { list: "01H" }), closed("02", { list: "01W" }), closed("03", {})],
      "list",
      lists,
    );

    expect(bands(rows)).toEqual(["Work", "Home", "No list"]);
  });

  it("ranks the quadrants the way the product does, with the unset before the minor", () => {
    const rows = shelved(
      [
        closed("01", { priority: "minor" }),
        closed("02", { priority: "unset" }),
        closed("03", { priority: "do" }),
      ],
      "quadrant",
      lists,
    );

    expect(bands(rows)).toEqual(["Do", "Unclassified", "Minor"]);
  });
});
