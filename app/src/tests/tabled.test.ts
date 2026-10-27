import { describe, expect, it, vi } from "vitest";
import { went } from "../ui/paging";
import {
  beside,
  cellOf,
  dragging,
  type Leaving,
  nthOf,
  placeOf,
  rowsShown,
  tableClicked,
  tabled,
  typing,
  viewed,
} from "../ui/tabled";

const TABLE = `<div class="table" data-at="0-90"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="select">Categoría</th><th scope="col" data-type="date">Cobro</th><th scope="col" data-type="check">Activa</th></tr></thead><tbody>
<tr><td>Spotify</td><td data-key="música">Música</td><td data-key="2026-11-14">2026-11-14</td><td data-key="1">✓</td></tr>
<tr><td>Netflix</td><td data-key="video">Video</td><td data-key="2026-01-03">2026-01-03</td><td data-key="0"></td></tr>
<tr><td>Duolingo</td><td data-key="educación">Educación</td><td data-amiss>pronto</td><td data-key="1">✓</td></tr>
</tbody></table></div>`;

const page = (html = TABLE): Document => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = `<article class="page">${html}</article>`;
  return made;
};

const dressed = (): Document => {
  const made = page();
  tabled(made);
  return made;
};

describe("a table dressed to be written in place", () => {
  it("gets a menu on each head, a grip on each row and a way to add one, once", () => {
    const made = dressed();
    tabled(made);
    const box = made.querySelector(".table") as HTMLElement;

    expect(box.classList.contains("dressed")).toBe(true);
    expect(box.querySelectorAll('th > button[data-act="column"]')).toHaveLength(4);
    expect(box.querySelector('th button[data-act="column"]')?.getAttribute("aria-label")).toBe(
      "Column menu: Servicio",
    );
    expect(
      Array.from(box.querySelectorAll<HTMLElement>(".row-grip")).map((one) => one.dataset.row),
    ).toEqual(["0", "1", "2"]);
    expect(box.querySelectorAll(".row-add")).toHaveLength(1);
    expect(box.querySelectorAll(".table-bar")).toHaveLength(1);
    expect(box.querySelector("th")?.getAttribute("aria-sort")).toBe("none");
  });

  it("knows a cell, a head and a click that is none of its own", () => {
    const made = dressed();
    const cell = cellOf(made, 0, 1, 2) as HTMLElement;
    const head = made.querySelectorAll("th")[1] as HTMLElement;

    expect(tableClicked(cell)).toEqual({ act: "cell", row: 1, column: 2 });
    expect(tableClicked(head)).toEqual({ act: "sort", row: -1, column: 1 });
    expect(tableClicked(page().querySelector("td") as Element)).toBeNull();
    expect(went(cell, false)).toMatchObject({
      kind: "table",
      at: [0, 90],
      act: "cell",
      row: 1,
      column: 2,
    });
    const grip = made.querySelector<HTMLElement>(".row-grip") as HTMLElement;
    expect(went(grip, false)).toMatchObject({ kind: "table", act: "row", row: 0, column: -1 });
    const more = made.querySelectorAll<HTMLElement>('button[data-act="column"]')[3];
    expect(went(more, false)).toMatchObject({ kind: "table", act: "column", column: 3 });
  });

  it("finds a table by where it sits in the text", () => {
    const made = dressed();
    expect(nthOf(made, [0, 90])).toBe(0);
    expect(nthOf(made, [1, 90])).toBe(-1);
    expect(placeOf(made, 0)).toEqual([0, 90]);
    expect(placeOf(made, 3)).toBeNull();
    expect(cellOf(made, 0, -1, 1)?.textContent).toContain("Categoría");
    expect(cellOf(made, 5, 0, 0)).toBeNull();
  });
});

describe("a table shown on screen only", () => {
  it("lays its rows in the order asked and says so above it", () => {
    const made = dressed();
    viewed(
      made,
      0,
      { sort: { column: 2, down: false, order: [1, 0, 2] }, filters: [] },
      "2026-10-27",
    );

    expect(rowsShown(made, 0)).toEqual([1, 0, 2]);
    expect(made.querySelectorAll("th")[2].getAttribute("aria-sort")).toBe("ascending");
    const bar = made.querySelector<HTMLElement>(".table-bar") as HTMLElement;
    expect(bar.hidden).toBe(false);
    expect(bar.textContent).toContain("Sorted by Cobro, on screen only");
    expect(bar.querySelector('[data-act="keep"]')).not.toBeNull();

    viewed(made, 0, undefined, "2026-10-27");
    expect(rowsShown(made, 0)).toEqual([0, 1, 2]);
    expect(bar.hidden).toBe(true);
  });

  // One filter per line: the column, what it keeps, then the rows left on screen.
  const FILTERS = `
0 has net        => 1
1 is música      => 0
3 yes            => 0 2
3 no             => 1
2 soon           => 0
2 past           => 1
`;

  it("hides the rows a filter leaves out and keeps the rest in place", () => {
    for (const line of FILTERS.trim().split("\n")) {
      const [asked, left] = line.split("=>").map((one) => one.trim());
      const [column, kind, said = ""] = asked.split(/\s+/);
      const made = dressed();
      viewed(
        made,
        0,
        {
          filters: [{ column: Number(column), kind: kind as never, said, label: `${kind}` }],
        },
        "2026-10-27",
      );
      expect(rowsShown(made, 0).join(" "), line).toBe(left);
      expect(made.querySelector('.table-chip [data-act="unfilter"]'), line).not.toBeNull();
    }
  });
});

describe("writing in a cell", () => {
  it("ends as the key that left it says, with what was written", () => {
    // One key per line, then how the writing ended.
    const KEYS = `
Enter           => enter
Escape          => cancel
Tab             => tab
shift+Tab       => back
alt+ArrowUp     => up
alt+ArrowDown   => down
`;
    for (const line of KEYS.trim().split("\n")) {
      const [key, how] = line.split("=>").map((one) => one.trim());
      const made = dressed();
      const cell = cellOf(made, 0, 0, 0) as HTMLElement;
      const done = vi.fn();
      const input = typing(cell, "Spotify", "Cell", done);
      input.value = "Tidal";
      const [mods, name] = key.includes("+") ? key.split("+") : ["", key];
      input.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: name,
          shiftKey: mods === "shift",
          altKey: mods === "alt",
          bubbles: true,
        }),
      );
      expect(done, line).toHaveBeenCalledWith("Tidal", how as Leaving);
      expect(cell.querySelector(".table-input"), line).toBeNull();
    }
  });

  it("leaves the cell when it loses the focus, and only once", () => {
    const made = dressed();
    const done = vi.fn();
    const input = typing(cellOf(made, 0, 0, 0) as HTMLElement, "x", "Cell", done);
    input.dispatchEvent(new FocusEvent("blur"));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    expect(done).toHaveBeenCalledTimes(1);
    expect(done).toHaveBeenCalledWith("x", "blur");
  });

  // From a row and column, how the writing ended, then where the caret goes next.
  const NEXT = `
0 0 tab    => 0 1
0 3 tab    => 1 0
2 3 tab    => new
0 1 back   => 0 0
1 0 back   => 0 3
0 0 back   => none
0 2 enter  => 1 2
2 2 enter  => none
1 1 blur   => none
`;

  it("moves on as a spreadsheet does, among the rows shown", () => {
    for (const line of NEXT.trim().split("\n")) {
      const [from, to] = line.split("=>").map((one) => one.trim());
      const [row, column, how] = from.split(/\s+/);
      const next = beside([0, 1, 2], 4, Number(row), Number(column), how as Leaving);
      const said = next === null ? "none" : next === "new" ? "new" : `${next.row} ${next.column}`;
      expect(said, line).toBe(to);
    }
    expect(beside([2, 0, 1], 4, 2, 0, "enter")).toEqual({ row: 0, column: 0 });
  });
});

describe("a row dragged by its grip", () => {
  it("lands on the row under the pointer, and a small slip is a click", () => {
    const made = dressed();
    const rows = Array.from(made.querySelectorAll<HTMLElement>("tbody > tr"));
    rows.forEach((one, at) => {
      one.getBoundingClientRect = () => ({ top: at * 40, bottom: at * 40 + 40 }) as DOMRect;
    });
    const moved = vi.fn();
    dragging(made, (box, from, to) => moved(box.dataset.at, from, to));
    const grip = made.querySelectorAll<HTMLElement>(".row-grip")[0];
    const pointer = (type: string, y: number, target: EventTarget = made) =>
      target.dispatchEvent(new MouseEvent(type, { bubbles: true, clientY: y, button: 0 }));

    pointer("pointerdown", 10, grip);
    pointer("pointermove", 95);
    pointer("pointerup", 95);
    expect(moved).toHaveBeenCalledWith("0-90", 0, 2);

    moved.mockClear();
    pointer("pointerdown", 10, grip);
    pointer("pointermove", 12);
    pointer("pointerup", 12);
    expect(moved).not.toHaveBeenCalled();
  });
});
