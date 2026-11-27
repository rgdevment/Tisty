import { describe, expect, it } from "vitest";
import { beside, EDGE, found, leadingOf, placement, stepped, typedTo } from "../ui/menuKeys";

const VIEW = { wide: 1200, tall: 800 };
const CARD = { wide: 210, tall: 300 };

describe("where a menu is placed", () => {
  it("hangs down from its anchor when it fits below", () => {
    expect(placement({ x: 100, y: 100, top: 80 }, CARD, VIEW)).toEqual({
      x: 100,
      top: 100,
      tall: 300,
    });
  });

  it("goes up at the bottom edge, standing on a bottom counted from the window's foot", () => {
    expect(placement({ x: 100, y: 780, top: 760 }, CARD, VIEW)).toEqual({
      x: 100,
      bottom: 800 - (760 - 4),
      tall: 300,
    });
  });

  it("stands on its line even with room below when asked to go up", () => {
    const placed = placement({ x: 100, y: 400, top: 380 }, CARD, VIEW, EDGE, true);

    expect(placed.bottom).toBe(800 - 376);
    expect(placed.top).toBeUndefined();
  });

  it("stays below and scrolls when there is fair room there", () => {
    expect(placement({ x: 100, y: 540, top: 520 }, CARD, VIEW)).toEqual({
      x: 100,
      top: 540,
      tall: 800 - EDGE - 540,
    });
  });

  it("goes above when whole and it fits there, rather than scrolling below", () => {
    const placed = placement({ x: 100, y: 540, top: 520 }, CARD, VIEW, EDGE, false, true);

    expect(placed.bottom).toBe(800 - (520 - 4));
    expect(placed.tall).toBe(300);
  });

  it("still goes below when whole and it fits below", () => {
    expect(placement({ x: 100, y: 100, top: 80 }, CARD, VIEW, EDGE, false, true)).toEqual({
      x: 100,
      top: 100,
      tall: 300,
    });
  });

  it("never climbs over the ceiling", () => {
    const placed = placement({ x: 100, y: 780, top: 300 }, CARD, VIEW, 91);

    expect(placed.tall).toBe(300 - 4 - 91);
    expect(VIEW.tall - (placed.bottom ?? 0) - placed.tall).toBe(91);
  });

  it("clamps x so the card stays inside the right edge and the left edge", () => {
    expect(placement({ x: 1190, y: 100 }, CARD, VIEW).x).toBe(1200 - 210 - EDGE);
    expect(placement({ x: -50, y: 100 }, CARD, VIEW).x).toBe(EDGE);
  });

  it("hangs from the anchor's right edge in the right half of the window", () => {
    expect(placement({ x: 900, y: 100, right: 920 }, CARD, VIEW).x).toBe(920 - 210);
    expect(placement({ x: 100, y: 100, right: 120 }, CARD, VIEW).x).toBe(100);
  });

  it("never grows past four fifths of the window", () => {
    expect(placement({ x: 100, y: 10 }, { wide: 210, tall: 5000 }, VIEW).tall).toBe(640);
  });

  it("pins itself to the window's foot when there is no room above or below", () => {
    const small = { wide: 1200, tall: 200 };

    expect(placement({ x: 100, y: 100, top: 90 }, CARD, small)).toEqual({
      x: 100,
      top: 200 - 160 - EDGE,
      tall: 160,
    });
  });
});

describe("where a list opens beside its row", () => {
  const row = { left: 100, right: 310, top: 50 };
  const small = { wide: 200, tall: 150 };

  it("goes to the right of the row, level with its top", () => {
    expect(beside(row, small, VIEW)).toEqual({ x: 310, top: 50, tall: 150 });
  });

  it("flips to the left when there is no room on the right", () => {
    expect(beside({ left: 1000, right: 1190, top: 50 }, small, VIEW).x).toBe(800);
  });

  it("stays inside the window when the left has no room either", () => {
    expect(beside({ left: 100, right: 1190, top: 50 }, small, VIEW).x).toBe(EDGE);
  });

  it("is lifted so it never runs off the bottom", () => {
    expect(beside({ ...row, top: 700 }, small, VIEW).top).toBe(800 - EDGE - 150);
    expect(beside({ ...row, top: -20 }, small, VIEW).top).toBe(EDGE);
  });

  it("is no taller than the window less its edges", () => {
    expect(beside(row, { wide: 200, tall: 2000 }, VIEW).tall).toBe(800 - 2 * EDGE);
  });
});

describe("the arrows over the rows", () => {
  it("wraps from the last row to the first and back", () => {
    expect(stepped(3, 2, 1)).toBe(0);
    expect(stepped(3, 0, -1)).toBe(2);
    expect(stepped(3, 0, 1)).toBe(1);
  });

  it("lands nowhere when there are no rows", () => {
    expect(stepped(0, 0, 1)).toBe(-1);
  });
});

describe("a letter typed over the rows", () => {
  const labels = ["Alpha", "Árbol", "Beta"];

  it("finds the next row starting with it, accents and case folded", () => {
    expect(typedTo(labels, 0, "a")).toBe(1);
    expect(typedTo(labels, 0, "B")).toBe(2);
  });

  it("wraps round from the end to the top", () => {
    expect(typedTo(labels, 1, "a")).toBe(0);
    expect(typedTo(labels, 2, "a")).toBe(0);
  });

  it("finds nothing when no row starts with it", () => {
    expect(typedTo(labels, 0, "z")).toBe(-1);
  });
});

describe("a row found by what was sought", () => {
  it("is found anywhere in its words, accents folded", () => {
    expect(found("Tabla grid", "abla")).toBe(true);
    expect(found("Árbol", "arbol")).toBe(true);
    expect(found("Tabla grid", "zzz")).toBe(false);
  });

  it("is always found when nothing is sought", () => {
    expect(found("Tabla", "")).toBe(true);
  });
});

describe("the row Enter takes", () => {
  it("prefers a word that begins with what was sought to one that only holds it", () => {
    const rows = [
      { words: "estable", opens: false },
      { words: "la tabla", opens: false },
    ];

    expect(leadingOf(rows, "tab")).toBe(1);
  });

  it("prefers a row that does the thing to one that opens a list", () => {
    const rows = [
      { words: "aviso", opens: true },
      { words: "aviso rapido", opens: false },
    ];

    expect(leadingOf(rows, "avi")).toBe(1);
  });

  it("opens a list when it is the only one beginning with the words", () => {
    const rows = [
      { words: "tabla", opens: false },
      { words: "pagina", opens: true },
    ];

    expect(leadingOf(rows, "pag")).toBe(1);
  });

  it("takes the first plain row when nothing is sought", () => {
    const rows = [
      { words: "aviso", opens: true },
      { words: "tabla", opens: false },
    ];

    expect(leadingOf(rows, "")).toBe(1);
  });

  it("falls back to the first plain row when no word begins with it", () => {
    const rows = [
      { words: "aviso", opens: true },
      { words: "tabla", opens: false },
    ];

    expect(leadingOf(rows, "zz")).toBe(1);
  });
});
