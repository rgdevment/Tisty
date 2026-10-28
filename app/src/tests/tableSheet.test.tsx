import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TableWent } from "../ui/paging";
import { rowsShown, tabled } from "../ui/tabled";
import { useTables } from "../ui/tableSheet";

const core = vi.hoisted(() => ({
  edit: vi.fn<(body: string, at: [number, number], ...edits: unknown[]) => Promise<string>>(),
  order:
    vi.fn<
      (body: string, at: [number, number], column: number, down: boolean) => Promise<number[]>
    >(),
  written:
    vi.fn<(body: string, at: [number, number]) => Promise<{ heads: string[]; rows: string[][] }>>(),
  canon: vi.fn<(words: string, typed: string) => Promise<string>>(),
}));

vi.mock("../tables", () => ({
  tableEdit: core.edit,
  tableOrder: core.order,
  tableWritten: core.written,
  tableCanon: core.canon,
  refused: (why: unknown) =>
    new Error(
      why === "moved"
        ? "The table changed meanwhile; try again."
        : "That change could not be made.",
    ),
}));

const BODY =
  "| Servicio | Precio {money CLP} |\n| --- | --: |\n| Spotify | 6490 |\n| Netflix | **9990** |\n";

const HTML = `<article class="page"><div class="table" data-at="0-80"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="money" data-currency="CLP">Precio</th></tr></thead><tbody>
<tr><td>Spotify</td><td data-key="6490">$6.490</td></tr>
<tr><td>Netflix</td><td data-key="9990">$9.990</td></tr>
</tbody></table></div></article>`;

type Seen = { onTable: (asked: TableWent) => void };

const made = (html = HTML) => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = html;
  tabled(page);
  const ways = {
    latest: { current: BODY },
    drawn: { current: BODY },
    whole: vi.fn(),
    leave: vi.fn(),
    markdown: vi.fn(),
    fail: vi.fn(),
    writing: false,
  };
  const seen: Seen = { onTable: () => {} };
  const Room = ({ turned }: { turned: number }) => {
    const tables = useTables({
      latest: ways.latest,
      drawn: ways.drawn,
      page: () => page,
      frame: () => ({ left: 100, top: 50 }) as DOMRect,
      whole: ways.whole,
      writing: () => ways.writing,
      leave: ways.leave,
      markdown: ways.markdown,
      fail: ways.fail,
      turned,
    });
    seen.onTable = tables.onTable;
    return <>{tables.menu}</>;
  };
  const view = render(<Room turned={1} />);
  const turn = (turned: number) => view.rerender(<Room turned={turned} />);
  return { page, ways, seen, turn };
};

const asked = (
  act_: TableWent["act"],
  row = -1,
  column = -1,
  extra: Partial<TableWent> = {},
): TableWent => ({
  kind: "table",
  at: [0, 80],
  act: act_,
  row,
  column,
  x: 10,
  y: 20,
  ...extra,
});

const settle = () => act(async () => {});

// The page is a document of its own, with no window for the testing library to find.
const press = (target: Element | null, key: string, altKey = false) =>
  target?.dispatchEvent(new KeyboardEvent("keydown", { key, altKey, bubbles: true }));

beforeEach(() => {
  core.canon.mockReset().mockImplementation((_, typed) => Promise.resolve(typed));
  core.edit.mockReset().mockResolvedValue("next body");
  core.order.mockReset().mockResolvedValue([1, 0]);
  core.written.mockReset().mockResolvedValue({
    heads: ["Servicio", "Precio"],
    rows: [
      ["Spotify", "6490"],
      ["Netflix", "**9990**"],
    ],
  });
});

describe("a table written in place", () => {
  it("opens a cell with its words, saves what was typed in the marks around it, then opens the cell below", async () => {
    const { page, ways, seen, turn } = made();
    seen.onTable(asked("cell", 1, 1));
    await settle();
    const input = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    expect(input.value).toBe("9990");
    expect(input.getAttribute("aria-label")).toBe("Cell in Precio");

    core.canon.mockImplementation((_, typed) => Promise.resolve(typed.replace(".", "")));
    input.value = "10.990";
    press(input, "Enter");
    await settle();
    // The words take the column's form before the marks go back around them.
    expect(core.canon).toHaveBeenCalledWith("money CLP", "10.990");
    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 1,
      column: 1,
      said: "**10990**",
    });
    expect(ways.whole).toHaveBeenCalledWith("next body");

    seen.onTable(asked("cell", 0, 0));
    await settle();
    const first = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    press(first, "Tab");
    await settle();
    expect(core.edit).toHaveBeenCalledTimes(1);
    expect(page.querySelector<HTMLInputElement>(".table-input")?.value).toBe("6490");

    turn(2);
    await settle();
    expect(core.written).toHaveBeenCalledTimes(3);
  });

  it("starts a new row past the last cell, and moves a row with Alt and an arrow", async () => {
    const { page, seen } = made();
    seen.onTable(asked("cell", 1, 1));
    await settle();
    press(page.querySelector(".table-input"), "Tab");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "rowAdd", at: 2 });

    seen.onTable(asked("cell", 1, 0));
    await settle();
    const input = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    input.value = "Max";
    press(input, "ArrowUp", true);
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(
      BODY,
      [0, 80],
      { kind: "cell", row: 1, column: 0, said: "Max" },
      { kind: "rowMove", from: 1, to: 0 },
    );
  });

  it("renames a head, keeping its type to the core", async () => {
    const { page, seen } = made();
    seen.onTable(asked("column", -1, 1));
    await settle();
    fireEvent.click(screen.getByRole("menuitem", { name: /Rename/ }));
    await settle();
    const input = page.querySelector<HTMLInputElement>("th .table-input") as HTMLInputElement;
    expect(input.value).toBe("Precio");
    input.value = "Costo";
    press(input, "Enter");
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "rename",
      column: 1,
      said: "Costo",
    });
  });

  it("sorts on screen in the core's order, turns it around, and saves it when asked", async () => {
    const { page, seen } = made();
    seen.onTable(asked("sort", -1, 1));
    await settle();
    expect(core.order).toHaveBeenLastCalledWith(BODY, [0, 80], 1, false);
    expect(rowsShown(page, 0)).toEqual([1, 0]);
    expect(page.querySelectorAll("th")[1].getAttribute("aria-sort")).toBe("ascending");

    seen.onTable(asked("sort", -1, 1));
    await settle();
    expect(core.order).toHaveBeenLastCalledWith(BODY, [0, 80], 1, true);

    seen.onTable(asked("keep"));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "sort",
      column: 1,
      down: true,
    });

    seen.onTable(asked("sort", -1, 0));
    await settle();
    seen.onTable(asked("unsort"));
    await settle();
    expect(rowsShown(page, 0)).toEqual([0, 1]);
  });

  it("opens a row's menu where its grip is, and a dragged row lands where it was let go", async () => {
    const { seen } = made();
    seen.onTable(asked("row", 0));
    await settle();
    expect(screen.queryByRole("menuitem", { name: /Move up/ })).toBeNull();
    fireEvent.click(screen.getByRole("menuitem", { name: /Move down/ }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "rowMove", from: 0, to: 1 });

    seen.onTable(asked("row", 1, -1, { to: 0 }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "rowMove", from: 1, to: 0 });

    seen.onTable(asked("rowAdd"));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "rowAdd", at: 2 });
  });

  it("filters from a column's menu and drops the filter from its label", async () => {
    const { page, seen } = made();
    seen.onTable(asked("column", -1, 0));
    await settle();
    fireEvent.click(screen.getByRole("menuitem", { name: /Filter/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Contains/ }));
    await settle();
    const input = page.querySelector<HTMLInputElement>(
      ".table-bar .table-input",
    ) as HTMLInputElement;
    input.value = "net";
    press(input, "Enter");
    await settle();
    expect(rowsShown(page, 0)).toEqual([1]);
    expect(page.querySelector(".table-chip")?.textContent).toContain("Servicio contains “net”");

    seen.onTable(asked("unfilter", -1, 0));
    await settle();
    expect(rowsShown(page, 0)).toEqual([0, 1]);
  });

  it("says why a change was refused, and waits while a block is being written", async () => {
    const { ways, seen } = made();
    core.edit.mockRejectedValue("moved");
    seen.onTable(asked("rowAdd"));
    await settle();
    expect(ways.fail).toHaveBeenCalledWith(new Error("The table changed meanwhile; try again."));

    ways.writing = true;
    seen.onTable(asked("cell", 0, 0));
    await settle();
    expect(ways.leave).toHaveBeenCalled();
    expect(core.written).not.toHaveBeenCalled();
  });

  it("changes nothing while the page on screen is a drawing behind the text", async () => {
    const { ways, seen } = made();
    ways.latest.current = `${BODY}\nMore.\n`;
    seen.onTable(asked("rowAdd"));
    seen.onTable(asked("cell", 0, 0));
    await settle();
    expect(core.edit).not.toHaveBeenCalled();
    expect(core.written).not.toHaveBeenCalled();
  });
});

describe("what a table shows, as the file changes under it", () => {
  it("opens the cell clicked while the one left behind was still being saved", async () => {
    const { page, ways, seen, turn } = made();
    let saved: (body: string) => void = () => {};
    core.edit.mockReturnValue(new Promise((done) => (saved = done)));
    seen.onTable(asked("cell", 0, 0));
    await settle();
    const input = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    input.value = "Tidal";
    input.dispatchEvent(new FocusEvent("blur"));
    seen.onTable(asked("cell", 1, 1));
    await settle();
    expect(core.written).toHaveBeenCalledTimes(1);

    await act(async () => saved("next body"));
    expect(ways.whole).toHaveBeenCalledWith("next body");
    turn(2);
    await settle();
    expect(core.written).toHaveBeenCalledTimes(2);
    expect(page.querySelectorAll("tbody tr")[1].querySelector(".table-input")).not.toBeNull();
  });

  it("drops a filter whose column is taken away, and every filter when a row is added", async () => {
    const { page, seen, turn } = made();
    seen.onTable(asked("column", -1, 1));
    await settle();
    fireEvent.click(screen.getByRole("menuitem", { name: /Delete column|Remove column/ }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], { kind: "columnRemove", column: 1 });

    seen.onTable(asked("column", -1, 0));
    await settle();
    fireEvent.click(screen.getByRole("menuitem", { name: /Filter/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Contains/ }));
    await settle();
    const field = page.querySelector<HTMLInputElement>(
      ".table-bar .table-input",
    ) as HTMLInputElement;
    field.value = "net";
    field.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(rowsShown(page, 0)).toEqual([1]);

    seen.onTable(asked("rowAdd"));
    await settle();
    turn(2);
    await settle();
    expect(page.querySelector(".table-chip")).toBeNull();
    expect(rowsShown(page, 0)).toEqual([0, 1]);
  });

  it("asks for the order once per sort and per drawing, not for each filter", async () => {
    const { page, seen, turn } = made();
    seen.onTable(asked("sort", -1, 1));
    await settle();
    seen.onTable(asked("column", -1, 0));
    await settle();
    fireEvent.click(screen.getByRole("menuitem", { name: /Filter/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Contains/ }));
    await settle();
    const field = page.querySelector<HTMLInputElement>(
      ".table-bar .table-input",
    ) as HTMLInputElement;
    field.value = "o";
    field.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(core.order).toHaveBeenCalledTimes(1);

    turn(2);
    await settle();
    expect(core.order).toHaveBeenCalledTimes(2);
  });

  it("starts over when a table is added or taken away elsewhere on the page", async () => {
    const { page, seen, turn } = made();
    seen.onTable(asked("sort", -1, 1));
    await settle();
    expect(rowsShown(page, 0)).toEqual([1, 0]);
    page
      .querySelector(".page")
      ?.insertAdjacentHTML("afterbegin", HTML.replace(/<\/?article[^>]*>/g, ""));
    tabled(page);
    turn(2);
    await settle();
    expect(core.order).toHaveBeenCalledTimes(1);
  });
});

const TYPED = `<article class="page"><div class="table" data-at="0-80"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="date">Cobro</th><th scope="col" data-type="rating">Nota</th><th scope="col" data-type="check">Activa</th><th scope="col" data-type="money" data-currency="CLP" data-total="sum">Precio</th></tr></thead><tbody>
<tr><td>Spotify</td><td data-key="2026-11-14">2026-11-14</td><td data-key="3">★★★☆☆</td><td data-key="1">✓</td><td data-key="6490">$6.490</td></tr>
</tbody></table></div></article>`;

describe("a typed cell written in place", () => {
  beforeEach(() => {
    core.written.mockResolvedValue({
      heads: ["Servicio", "Cobro", "Nota", "Activa", "Precio"],
      rows: [["Spotify", "2026-11-14", "3", "[x]", "6490"]],
    });
  });

  it("writes a star or the switch with one press", async () => {
    const { seen } = made(TYPED);
    seen.onTable(asked("star", 0, 2, { to: 5 }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 2,
      said: "5",
    });
    seen.onTable(asked("star", 0, 2, { to: 0 }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 2,
      said: "",
    });
    seen.onTable(asked("toggle", 0, 3, { to: 0 }));
    await settle();
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 3,
      said: "[ ]",
    });
  });

  it("opens a date in its own picker and writes what is said there in the file's form", async () => {
    const { page, seen } = made(TYPED);
    core.canon.mockResolvedValue("2026-10-30");
    seen.onTable(asked("cell", 0, 1));
    await settle();
    expect(page.querySelector(".table-input")).toBeNull();
    const field = screen.getByRole("textbox", { name: /Or type it/ });
    (field as HTMLInputElement).value = "el viernes";
    fireEvent.keyDown(field, { key: "Enter" });
    await settle();
    expect(core.canon).toHaveBeenCalledWith("date", "el viernes");
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 1,
      said: "2026-10-30",
    });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("puts money typed in the cell into its currency's plain form before writing it", async () => {
    const { page, seen } = made(TYPED);
    core.canon.mockResolvedValue("7990");
    seen.onTable(asked("cell", 0, 4));
    await settle();
    const input = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    input.value = "$7.990";
    press(input, "Enter");
    await settle();
    expect(core.canon).toHaveBeenCalledWith("money CLP sum", "$7.990");
    expect(core.edit).toHaveBeenLastCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 0,
      column: 4,
      said: "7990",
    });
  });
});
