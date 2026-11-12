import { describe, expect, it } from "vitest";
import {
  added,
  boardOf,
  bornHeading,
  COLOURS,
  colourOf,
  dropped,
  introOf,
  ordered,
  rewritten,
  saidWhen,
  shapeOf,
  shifted,
  stampOf,
  without,
} from "../notes";

const BRAINSTORM = `---
view: notes
brainstorm: on
---

# Lluvia de ideas

## 2026-11-10 18:40

vecinos en el panel

## 2026-11-11 09:12

llamar al vet
y pedir la cartilla
`;

describe("a document read as notes", () => {
  it("keeps everything above the first section as its head, and each section is a note", () => {
    const board = boardOf(BRAINSTORM);

    expect(board.head).toBe("---\nview: notes\nbrainstorm: on\n---\n\n# Lluvia de ideas");
    expect(board.notes).toEqual([
      { heading: "2026-11-10 18:40", text: "vecinos en el panel" },
      { heading: "2026-11-11 09:12", text: "llamar al vet\ny pedir la cartilla" },
    ]);
  });

  it("does not take a heading inside a code fence for a new note", () => {
    const board = boardOf("# Ideas\n\n## uno\n\n```md\n## no es nota\n```\n");

    expect(board.notes).toHaveLength(1);
    expect(board.notes[0].text).toContain("## no es nota");
  });

  it("changes, drops and adds one note and leaves the rest of the file as it was", () => {
    const changed = rewritten(BRAINSTORM, 0, "vecinos, no grafo");
    expect(boardOf(changed).notes[0].text).toBe("vecinos, no grafo");
    expect(boardOf(changed).notes[1]).toEqual(boardOf(BRAINSTORM).notes[1]);
    expect(changed.startsWith("---\nview: notes\n")).toBe(true);

    const gone = dropped(BRAINSTORM, 0);
    expect(boardOf(gone).notes.map((one) => one.heading)).toEqual(["2026-11-11 09:12"]);

    const more = added(BRAINSTORM, "2026-11-11 10:00");
    expect(boardOf(more).notes[2]).toEqual({ heading: "2026-11-11 10:00", text: "" });
  });

  it("puts the notes in a new order and keeps every word of each", () => {
    const flipped = ordered(BRAINSTORM, [1, 0]);

    expect(boardOf(flipped).notes).toEqual([...boardOf(BRAINSTORM).notes].reverse());
    expect(boardOf(flipped).head).toBe(boardOf(BRAINSTORM).head);
  });

  it("moves one place to another and leaves the rest in their order", () => {
    expect(shifted([0, 1, 2, 3], 3, 0)).toEqual([3, 0, 1, 2]);
    expect(shifted([0, 1, 2, 3], 0, 2)).toEqual([1, 2, 0, 3]);
  });

  it("takes an idea off by what it says, wherever the sheet has moved it", () => {
    const [first] = boardOf(BRAINSTORM).notes;
    const shuffled = ordered(BRAINSTORM, [1, 0]);

    const left = boardOf(without(shuffled, first)).notes;

    expect(left.map((one) => one.text)).toEqual(["llamar al vet\ny pedir la cartilla"]);
    expect(without(BRAINSTORM, { heading: "nunca", text: "estuvo" })).toBe(BRAINSTORM);
  });

  it("leaves every other note byte for byte, its line endings included", () => {
    const windows = "# Ideas\r\n\r\n## uno\r\n\r\nprimera  \r\n\r\n\r\n## dos\r\n\r\nsegunda\r\n";

    const changed = rewritten(windows, 1, "segunda, mejor");

    expect(changed).toBe(
      "# Ideas\r\n\r\n## uno\r\n\r\nprimera  \r\n\r\n\r\n## dos\r\n\r\nsegunda, mejor\r\n",
    );
  });

  it("shows what the head says below its title, without its properties", () => {
    expect(introOf("---\nview: notes\n---\n\n# Ideas\n\nLo que se me ocurre.\n\n## uno\n\nx")).toBe(
      "Lo que se me ocurre.",
    );
    expect(introOf(BRAINSTORM)).toBe("");
  });

  it("reads a CRLF file the same as an LF one", () => {
    expect(boardOf(BRAINSTORM.replace(/\n/g, "\r\n"))).toEqual(boardOf(BRAINSTORM));
  });
});

describe("a note's look", () => {
  it("is the same every time for the same note, and editing its words never repaints it", () => {
    expect(shapeOf("2026-11-11 09:12", 0)).toEqual(shapeOf("2026-11-11 09:12", 0));
  });

  it("differs between notes that came in the same minute", () => {
    const shapes = [0, 1, 2, 3, 4, 5].map((nth) =>
      JSON.stringify(shapeOf("2026-11-12 00:10", nth)),
    );
    expect(new Set(shapes).size).toBe(shapes.length);
  });

  it("spreads its colours and its tape or pin across many notes", () => {
    const many = Array.from({ length: 60 }, (_, nth) => shapeOf("2026-11-12 00:10", nth));
    expect(new Set(many.map((one) => one.hue)).size).toBe(12);
    expect(new Set(many.map((one) => one.hold)).size).toBe(4);
    for (const one of many) {
      expect(Math.abs(one.tilt)).toBeLessThanOrEqual(7);
      expect(Math.abs(one.nudge)).toBeLessThanOrEqual(3);
    }
  });
});

describe("a note's colour", () => {
  it("is chosen when the idea is born and written in its heading", () => {
    const heading = bornHeading("# Ideas", new Date(2026, 10, 12, 0, 19), 0.7);

    expect(heading).toMatch(/^2026-11-12 00:19 · [a-z]+$/);
    expect(colourOf(heading)).toBeGreaterThanOrEqual(0);
  });

  it("is never the colour of the idea just before it", () => {
    let body = "# Ideas";
    for (const roll of [0, 0, 0.01, 0.5, 0.5, 0.99, 0.99]) {
      const heading = bornHeading(body, new Date(2026, 10, 12, 0, 19), roll);
      const notes = boardOf(body).notes;
      if (notes.length)
        expect(colourOf(heading)).not.toBe(colourOf(notes[notes.length - 1].heading));
      body = added(body, heading);
    }
  });

  it("is read back by name, and an unknown one counts as none", () => {
    expect(colourOf(`2026-11-12 00:19 · ${COLOURS[3]}`)).toBe(3);
    expect(colourOf("2026-11-12 00:19 · plaid")).toBe(-1);
    expect(colourOf("2026-11-12 00:19")).toBe(-1);
  });
});

describe("a note's time", () => {
  it("is written as the core writes it and shown in the person's words", () => {
    expect(stampOf(new Date(2026, 10, 11, 9, 7))).toBe("2026-11-11 09:07");
    expect(saidWhen("2026-11-11 09:07", "es")).toMatch(/^11 nov\.? · 09:07$/);
    expect(saidWhen("2026-11-11 09:07 · blue", "es")).toMatch(/^11 nov\.? · 09:07$/);
    expect(saidWhen("una idea sin fecha", "es")).toBe("una idea sin fecha");
  });
});
