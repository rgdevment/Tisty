import { describe, expect, it } from "vitest";
import type { DateSpec, Repeat } from "../core";
import { bandOf, cadence, weigh, whenLabel } from "../format";
import { fields, withLandscape, withPaper, withSheets } from "../front";
import { previewOf, weighed } from "../previews";

const day = (at: string): DateSpec => ({
  at,
  tz: "America/Santiago",
  floating: false,
  has_time: false,
});

const now = new Date("2026-06-10T09:00:00");

describe("the words the window says about dates", () => {
  it("names the bands tasks are grouped under", () => {
    const bands = [undefined, "2026-06-09", "2026-06-10", "2026-06-11", "2026-07-02"].map((at) =>
      bandOf(at ? day(at) : undefined, now),
    );
    expect(bands).toMatchInlineSnapshot(`
      [
        "Someday",
        "Overdue",
        "Today",
        "Tomorrow",
        "Thu, Jul 2",
      ]
    `);
  });

  it("says when a task is due in every nearness", () => {
    const labels = ["2026-06-08", "2026-06-09", "2026-06-10", "2026-06-11", "2026-06-20"].map(
      (at) => whenLabel(day(at), now),
    );
    expect(labels).toMatchInlineSnapshot(`
      [
        "Mon, Jun 8",
        "yesterday",
        "today",
        "tomorrow",
        "Sat, Jun 20",
      ]
    `);
  });

  it("says how a task repeats", () => {
    const each = (every: number, unit: Repeat["each"]["unit"], until?: string) =>
      cadence({ each: { every, unit }, until } as Repeat);
    expect([
      each(1, "day"),
      each(3, "day"),
      each(1, "week"),
      each(2, "week"),
      each(1, "month", "2026-12-31"),
      each(5, "year"),
    ]).toMatchInlineSnapshot(`
      [
        "every day",
        "every 3 days",
        "every week",
        "every 2 weeks",
        "every month until Dec 31",
        "every 5 years",
      ]
    `);
  });
});

describe("the words the window says about sizes and files", () => {
  it("weighs bytes the same way in both places", () => {
    const sizes = [0, 999, 1000, 999_950, 1_700_000, 2_100_000_000, -5, Number.NaN];
    expect(sizes.map((one) => `${weigh(one)} | ${weighed(one)}`)).toMatchInlineSnapshot(`
      [
        "0 B | 0 B",
        "999 B | 999 B",
        "1.0 kB | 1.0 kB",
        "1.0 MB | 1.0 MB",
        "1.7 MB | 1.7 MB",
        "2.1 GB | 2.1 GB",
        "0 B | 0 B",
        "— | —",
      ]
    `);
  });

  it("tells what a link in a note would open", () => {
    const links = [
      "tisty:doc/01JABC",
      "https://example.org/a/b",
      "https://example.org/clip.mp4",
      "attachments/ab/plan-0006bf09.pdf",
      "song.mp3",
      "notes/readme",
      "",
    ];
    expect(Object.fromEntries(links.map((one) => [one, previewOf(one)]))).toMatchSnapshot();
  });
});

describe("the front matter of a document", () => {
  const body = [
    "---",
    "title: Plan",
    "tags: [work, 'home office']",
    "aliases:",
    "  - one",
    "  - two",
    "# a comment",
    "paper: a4",
    "---",
    "",
    "# Plan",
  ].join("\n");

  it("lists its fields in order", () => {
    expect(fields(body)).toMatchSnapshot();
  });

  it("writes the page layout back into the block", () => {
    const laid = withSheets(withLandscape(withPaper(body, "letter"), true), true);
    expect(laid).toMatchSnapshot();
  });
});
