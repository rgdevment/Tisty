import { describe, expect, it } from "vitest";
import { added, boardOf, dropped, rewritten, saidWhen, shapeOf, stampOf } from "../notes";

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

  it("reads a CRLF file the same as an LF one", () => {
    expect(boardOf(BRAINSTORM.replace(/\n/g, "\r\n"))).toEqual(boardOf(BRAINSTORM));
  });
});

describe("a note's look", () => {
  it("is the same every time for the same words, and changes with them", () => {
    const one = { heading: "2026-11-11 09:12", text: "llamar al vet" };

    expect(shapeOf(one)).toEqual(shapeOf({ ...one }));
    const shapes = new Set(
      ["a", "b", "c", "d", "e", "f"].map((text) => JSON.stringify(shapeOf({ ...one, text }))),
    );
    expect(shapes.size).toBeGreaterThan(1);
  });

  it("stays within a gentle tilt and the five colours", () => {
    for (const text of ["uno", "dos", "tres", "cuatro", "cinco", "seis", "siete"]) {
      const shape = shapeOf({ heading: "x", text });
      expect(Math.abs(shape.tilt)).toBeLessThanOrEqual(4);
      expect(shape.hue).toBeGreaterThanOrEqual(0);
      expect(shape.hue).toBeLessThan(5);
    }
  });
});

describe("a note's time", () => {
  it("is written as the core writes it and shown in the person's words", () => {
    expect(stampOf(new Date(2026, 10, 11, 9, 7))).toBe("2026-11-11 09:07");
    expect(saidWhen("2026-11-11 09:07", "es")).toMatch(/^11 nov\.? · 09:07$/);
    expect(saidWhen("una idea sin fecha", "es")).toBe("una idea sin fecha");
  });
});
