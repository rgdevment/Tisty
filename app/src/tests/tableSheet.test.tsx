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
}));

vi.mock("../tables", () => ({
  tableEdit: core.edit,
  tableOrder: core.order,
  tableWritten: core.written,
}));

const BODY =
  "| Servicio | Precio {money CLP} |\n| --- | --: |\n| Spotify | 6490 |\n| Netflix | **9990** |\n";

const HTML = `<article class="page"><div class="table" data-at="0-80"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="money" data-currency="CLP">Precio</th></tr></thead><tbody>
<tr><td>Spotify</td><td data-key="6490">$6.490</td></tr>
<tr><td>Netflix</td><td data-key="9990">$9.990</td></tr>
</tbody></table></div></article>`;

type Seen = { onTable: (asked: TableWent) => void };

const made = () => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = HTML;
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
  it("opens a cell with its Markdown and saves what was typed, then opens the cell below", async () => {
    const { page, ways, seen, turn } = made();
    seen.onTable(asked("cell", 1, 1));
    await settle();
    const input = page.querySelector<HTMLInputElement>(".table-input") as HTMLInputElement;
    expect(input.value).toBe("**9990**");
    expect(input.getAttribute("aria-label")).toBe("Cell in Precio");

    input.value = "10990";
    press(input, "Enter");
    await settle();
    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 80], {
      kind: "cell",
      row: 1,
      column: 1,
      said: "10990",
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
