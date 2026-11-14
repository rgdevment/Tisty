import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TableWent } from "../ui/paging";
import { tableClicked, tabled } from "../ui/tabled";
import { useTables } from "../ui/tableSheet";

const core = vi.hoisted(() => ({
  edit: vi.fn<(body: string, at: [number, number], ...edits: unknown[]) => Promise<string>>(),
  order: vi.fn<() => Promise<number[]>>(),
  written: vi.fn<() => Promise<{ heads: string[]; rows: string[][] }>>(),
  canon: vi.fn<(words: string, typed: string) => Promise<string>>(),
}));

vi.mock("../tables", () => ({
  tableEdit: core.edit,
  tableOrder: core.order,
  tableWritten: core.written,
  tableCanon: core.canon,
  refused: () => new Error("That change could not be made."),
}));

const BODY = "| A | B | C |\n| --- | --- | --- |\n| a1 | b1 | c1 |\n| a2 | b2 | c2 |\n";

const table = (heads: string[]) =>
  `<article class="page"><div class="table" data-at="0-80"><table><thead><tr>${heads
    .map((one) => `<th scope="col">${one}</th>`)
    .join("")}</tr></thead><tbody>
<tr><td>a1</td><td>b1</td><td>c1</td></tr>
<tr><td>a2</td><td>b2</td><td>c2</td></tr>
</tbody></table></div></article>`;

const NAMED = table(["A", "B", "C"]);
const NAMELESS = table(["A", "B", ""]);

type Seen = { onTable: (asked: TableWent) => void };

const Room = ({
  page,
  seen,
  turned,
  whole,
  passed,
}: {
  page: Document;
  seen: Seen;
  turned: number;
  whole: (next: string) => void;
  passed?: (end: number) => void;
}) => {
  const tables = useTables({
    latest: { current: BODY },
    drawn: { current: BODY },
    page: () => page,
    frame: () => ({ left: 0, top: 0 }) as DOMRect,
    whole,
    passed,
    writing: () => false,
    leave: () => {},
    markdown: () => {},
    fail: () => {},
    turned,
  });
  seen.onTable = tables.onTable;
  return <>{tables.menu}</>;
};

const made = (html = NAMED) => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = html;
  tabled(page);
  const seen: Seen = { onTable: () => {} };
  const whole = vi.fn();
  const passed = vi.fn();
  const view = render(<Room page={page} seen={seen} turned={1} whole={whole} passed={passed} />);
  const turn = (turned: number) =>
    view.rerender(<Room page={page} seen={seen} turned={turned} whole={whole} passed={passed} />);
  return { page, seen, whole, turn, passed };
};

const asked = (act_: TableWent["act"], row: number, column: number): TableWent => ({
  kind: "table",
  at: [0, 80],
  act: act_,
  row,
  column,
  x: 10,
  y: 20,
});

const settle = () => act(async () => {});

const press = (target: Element | null, key: string, more: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...more });
  target?.dispatchEvent(event);
  return event;
};

const input = (page: Document) => page.querySelector<HTMLInputElement>(".table-input");

const cellOf = (page: Document, row: number, column: number) =>
  page.querySelectorAll("tbody tr")[row]?.querySelectorAll("td")[column];

const open = async (seen: Seen, row: number, column: number) => {
  seen.onTable(asked("cell", row, column));
  await settle();
};

beforeEach(() => {
  core.canon.mockReset().mockImplementation((_, typed) => Promise.resolve(typed));
  core.edit.mockReset().mockResolvedValue("next body");
  core.order.mockReset().mockResolvedValue([0, 1]);
  core.written.mockReset().mockResolvedValue({
    heads: ["A", "B", "C"],
    rows: [
      ["a1", "b1", "c1"],
      ["a2", "b2", "c2"],
    ],
  });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("a click on a head of a table", () => {
  it("sorts by a head that has a name", () => {
    const { page } = made(NAMELESS);
    const head = page.querySelectorAll("th")[0] as Element;
    expect(tableClicked(head)).toEqual({ act: "sort", row: -1, column: 0 });
  });

  it("opens a head with no name to be named instead of sorting", () => {
    const { page } = made(NAMELESS);
    const head = page.querySelectorAll("th")[2] as Element;
    expect(tableClicked(head)).toEqual({ act: "cell", row: -1, column: 2 });
  });

  it("opens a head holding only spaces to be named too", () => {
    const { page } = made(table(["A", "B", "   "]));
    const head = page.querySelectorAll("th")[2] as Element;
    expect(tableClicked(head)?.act).toBe("cell");
  });

  it("opens the head with no name in a field of its own, empty and labelled as a title", async () => {
    const { page, seen } = made(NAMELESS);
    core.written.mockResolvedValue({ heads: ["A", "B", ""], rows: [] });
    await open(seen, -1, 2);
    const field = input(page);
    expect(field?.closest("th")?.cellIndex).toBe(2);
    expect(field?.value).toBe("");
    expect(field?.getAttribute("aria-label")).toBe("Column title");
  });

  it("names a head with no name and writes it as a rename", async () => {
    const { page, seen } = made(NAMELESS);
    core.written.mockResolvedValue({ heads: ["A", "B", ""], rows: [] });
    await open(seen, -1, 2);
    const field = input(page) as HTMLInputElement;
    field.value = "Total";
    press(field, "Enter");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "rename",
      column: 2,
      said: "Total",
    });
  });

  it("leaves a head with no name as it is when nothing is written and Escape is pressed", async () => {
    const { page, seen } = made(NAMELESS);
    await open(seen, -1, 2);
    press(input(page), "Escape");
    await settle();
    expect(core.edit).not.toHaveBeenCalled();
    expect(input(page)).toBeNull();
  });
});

describe("Tab and Shift+Tab across the heads", () => {
  it("goes from a head to the next head", async () => {
    const { page, seen } = made();
    await open(seen, -1, 0);
    press(input(page), "Tab");
    await settle();
    expect(input(page)?.closest("th")?.cellIndex).toBe(1);
    expect(core.edit).not.toHaveBeenCalled();
  });

  it("goes from the last head to the first cell of the first row", async () => {
    const { page, seen } = made();
    await open(seen, -1, 2);
    press(input(page), "Tab");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 0));
  });

  it("writes the new name of a head and opens the next head once the page is drawn again", async () => {
    const { page, seen, turn } = made();
    await open(seen, -1, 0);
    const field = input(page) as HTMLInputElement;
    field.value = "Servicio";
    press(field, "Tab");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "rename",
      column: 0,
      said: "Servicio",
    });
    turn(2);
    await settle();
    expect(input(page)?.closest("th")?.cellIndex).toBe(1);
  });

  it("goes back from a head to the head before it with Shift+Tab", async () => {
    const { page, seen } = made();
    await open(seen, -1, 1);
    press(input(page), "Tab", { shiftKey: true });
    await settle();
    expect(input(page)?.closest("th")?.cellIndex).toBe(0);
  });

  it("goes nowhere back from the first head", async () => {
    const { page, seen } = made();
    await open(seen, -1, 0);
    press(input(page), "Tab", { shiftKey: true });
    await settle();
    expect(input(page)).toBeNull();
  });

  it("goes from the first cell of the first row nowhere with Shift+Tab", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab", { shiftKey: true });
    await settle();
    expect(input(page)).toBeNull();
  });

  it("goes from the first cell of a row to the last cell of the row above with Shift+Tab", async () => {
    const { page, seen } = made();
    await open(seen, 1, 0);
    press(input(page), "Tab", { shiftKey: true });
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 2));
  });

  it("goes from the last cell of a row to the first cell of the next row with Tab", async () => {
    const { page, seen } = made();
    await open(seen, 0, 2);
    press(input(page), "Tab");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 1, 0));
  });

  it("starts a new row from the last cell of the last row with Tab", async () => {
    const { page, seen } = made();
    await open(seen, 1, 2);
    press(input(page), "Tab");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "rowAdd", at: 2 });
  });

  it("opens the cell below with Enter, and nothing from the last row", async () => {
    const { page, seen } = made();
    await open(seen, 0, 1);
    press(input(page), "Enter");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 1, 1));
    press(input(page), "Enter");
    await settle();
    expect(input(page)).toBeNull();
  });

  it("leaves the table with Enter on the last row, to a line begun under it", async () => {
    const { page, seen, passed } = made();
    await open(seen, 0, 1);
    press(input(page), "Enter");
    await settle();
    expect(passed).not.toHaveBeenCalled();
    press(input(page), "Enter");
    await settle();
    expect(passed).toHaveBeenCalledTimes(1);
  });

  it("opens the first row with Enter from a head", async () => {
    const { page, seen } = made(NAMELESS);
    await open(seen, -1, 2);
    press(input(page), "Enter");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 2));
  });
});

describe("keys typed while the next cell is saved and opened", () => {
  const typed = (page: Document, ...keys: string[]) =>
    keys.map((one) =>
      page.body.dispatchEvent(
        new KeyboardEvent("keydown", { key: one, bubbles: true, cancelable: true }),
      ),
    );

  it("holds them and lands them in the cell that opens", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    typed(page, "x", "y");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 1));
    expect(input(page)?.value).toBe("xy");
  });

  it("holds back the keys from the page while they wait", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    const event = press(page.body, "x");
    expect(event.defaultPrevented).toBe(true);
    await settle();
  });

  it("puts the caret after the held words", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    typed(page, "x", "y");
    await settle();
    expect(input(page)?.selectionStart).toBe(2);
    expect(input(page)?.selectionEnd).toBe(2);
  });

  it("writes the held words into the cell, replacing what it held", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    typed(page, "n", "e", "w");
    await settle();
    press(input(page), "Enter");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 1,
      said: "new",
    });
  });

  it("replays a Tab held among them, in order, into the cell after", async () => {
    const { page, seen, turn } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    typed(page, "x", "Tab", "y");
    await settle();
    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 1,
      said: "x",
    });
    turn(2);
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 2));
    expect(input(page)?.value).toBe("y");
  });

  it("replays an Enter held among them, in order, into the cell below", async () => {
    const { page, seen, turn } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    typed(page, "x", "Enter", "y");
    await settle();
    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 1,
      said: "x",
    });
    turn(2);
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 1, 1));
    expect(input(page)?.value).toBe("y");
  });

  it("replays several Tabs one after the other", async () => {
    const { page, seen, turn } = made();
    await open(seen, 0, 0);
    press(input(page), "Enter");
    typed(page, "a", "Tab", "b", "Tab", "c");
    await settle();
    turn(2);
    await settle();
    turn(3);
    await settle();
    expect(core.edit).toHaveBeenCalledTimes(2);
    expect(input(page)?.closest("td")).toBe(cellOf(page, 1, 2));
    expect(input(page)?.value).toBe("c");
  });

  it("leaves Shift+Tab out of what is held", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    press(page.body, "Tab", { shiftKey: true });
    typed(page, "x");
    await settle();
    expect(input(page)?.value).toBe("x");
  });

  it("does not hold a key pressed with Ctrl", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    const event = press(page.body, "z", { ctrlKey: true });
    expect(event.defaultPrevented).toBe(false);
    await settle();
    expect(input(page)?.value).toBe("b1");
  });

  it("does not hold an arrow or a function key", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    expect(press(page.body, "ArrowLeft").defaultPrevented).toBe(false);
    expect(press(page.body, "F2").defaultPrevented).toBe(false);
    await settle();
  });

  it("does not hold what is typed in a cell's own field", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    await settle();
    const event = press(input(page), "q");
    expect(event.defaultPrevented).toBe(false);
  });

  it("does not hold keys after a cell is left with Escape", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Escape");
    await settle();
    expect(press(page.body, "x").defaultPrevented).toBe(false);
  });

  it("does not hold keys after a cell is left with a click elsewhere", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    input(page)?.dispatchEvent(new FocusEvent("blur"));
    await settle();
    expect(press(page.body, "x").defaultPrevented).toBe(false);
  });

  it("lets the keys go after three seconds", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    vi.advanceTimersByTime(3001);
    expect(press(page.body, "x").defaultPrevented).toBe(false);
  });

  it("holds the keys of the next cell too while it is saved and the one after is opened", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab");
    await settle();
    press(input(page), "Tab");
    typed(page, "z");
    await settle();
    expect(input(page)?.closest("td")).toBe(cellOf(page, 0, 2));
    expect(input(page)?.value).toBe("z");
  });

  it("lets keys go to the page when the cell left was the last one and nothing opens", async () => {
    const { page, seen } = made();
    await open(seen, 1, 0);
    press(input(page), "Enter");
    await settle();
    expect(input(page)).toBeNull();
    expect(press(page.body, "x").defaultPrevented).toBe(false);
  });

  it("lets keys go to the page when Shift+Tab leaves the first cell and nothing opens", async () => {
    const { page, seen } = made();
    await open(seen, 0, 0);
    press(input(page), "Tab", { shiftKey: true });
    await settle();
    expect(input(page)).toBeNull();
    expect(press(page.body, "x").defaultPrevented).toBe(false);
  });
});
