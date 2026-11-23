import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import type { Seat } from "../ui/pageWriter";
import type { TableWent } from "../ui/paging";
import { tabled } from "../ui/tabled";
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

const BODY =
  "| Servicio | Tipo {select} | Precio {money CLP} |\n| --- | --- | --: |\n| Spotify | Música | 6490 |\n| Netflix | Video | 9990 |\n";

const HTML = `<article class="page"><div class="table" data-at="0-120"><table><thead><tr><th scope="col">Servicio</th><th scope="col" data-type="select">Tipo</th><th scope="col" data-type="money" data-currency="CLP">Precio</th></tr></thead><tbody>
<tr><td>Spotify</td><td>Música</td><td data-key="6490">$6.490</td></tr>
<tr><td>Netflix</td><td>Video</td><td data-key="9990">$9.990</td></tr>
</tbody></table></div></article>`;

type Held = {
  page: Document;
  text: { current: string };
  behind: string | null;
  seat: { now: Seat };
  lost: ReturnType<typeof vi.fn<(notice: string) => void>>;
  whole: ReturnType<typeof vi.fn<(next: string) => void>>;
  passed: ReturnType<
    typeof vi.fn<(end: number, at: readonly [number, number], typed?: string) => boolean>
  >;
  fail: ReturnType<typeof vi.fn<(problem: unknown) => void>>;
  kept: string[] | null;
  told: unknown[];
  onTable: (asked: TableWent) => void;
};

const Room = ({ held, turned }: { held: Held; turned: number }) => {
  const tables = useTables({
    latest: held.text,
    drawn: {
      get current() {
        return held.behind ?? held.text.current;
      },
    },
    page: () => held.page,
    frame: () => ({ left: 0, top: 0 }) as DOMRect,
    whole: (next) => {
      held.text.current = next;
      held.whole(next);
    },
    passed: held.passed,
    writing: () => false,
    leave: () => {},
    markdown: () => {},
    fail: (problem) => {
      held.told.push(problem);
      held.fail(problem);
    },
    seat: () => held.seat.now,
    lost: (notice) => {
      held.told.push(notice);
      held.lost(notice);
    },
    aside: () => {
      const kept = held.kept;
      return kept && ((words: string) => kept.push(words));
    },
    turned,
  });
  held.onTable = tables.onTable;
  return <>{tables.menu}</>;
};

const made = () => {
  const page = document.implementation.createHTMLDocument("");
  page.body.innerHTML = HTML;
  tabled(page);
  const held: Held = {
    page,
    text: { current: BODY },
    behind: null,
    seat: { now: { current: null } },
    lost: vi.fn(),
    kept: null,
    whole: vi.fn(),
    passed: vi.fn(() => true),
    fail: vi.fn(),
    told: [],
    onTable: () => {},
  };
  const view = render(<Room held={held} turned={1} />);
  const turn = (turned: number) => view.rerender(<Room held={held} turned={turned} />);
  const moved = () => {
    held.seat.now = { current: null };
  };
  return { ...held, held, turn, moved };
};

const asked = (act_: TableWent["act"], row: number, column: number): TableWent => ({
  kind: "table",
  at: [0, 120],
  act: act_,
  row,
  column,
  x: 10,
  y: 20,
});

const settle = () => act(async () => {});

const press = (target: Element | null, key: string) =>
  target?.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

const field = (page: Document) => page.querySelector<HTMLInputElement>(".table-input");

const held = <T,>() => {
  const said: { answer: (one: T) => void } = { answer: () => {} };
  const asked = new Promise<T>((done) => {
    said.answer = done;
  });
  return { said, asked };
};

beforeEach(() => {
  core.canon.mockReset().mockImplementation((_, typed) => Promise.resolve(typed));
  core.edit.mockReset().mockResolvedValue("next body");
  core.order.mockReset().mockResolvedValue([0, 1]);
  core.written.mockReset().mockResolvedValue({
    heads: ["Servicio", "Tipo", "Precio"],
    rows: [
      ["Spotify", "Música", "6490"],
      ["Netflix", "Video", "9990"],
    ],
  });
});

describe("a cell left after its page gave way to another document", () => {
  it("asks no change once its words take their form, and says it came too late", async () => {
    const { held: room, page, moved, lost } = made();
    room.onTable(asked("cell", 1, 2));
    await settle();
    const form = held<string>();
    core.canon.mockReturnValue(form.asked);
    const input = field(page) as HTMLInputElement;
    input.value = "10990";
    press(input, "Enter");

    moved();
    form.said.answer("10990");
    await settle();

    expect(core.edit).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("tableLost"));
  });

  it("asks no change of a title left once the page moved on", async () => {
    const { held: room, page, moved, lost } = made();
    room.onTable(asked("cell", -1, 0));
    await settle();
    const input = field(page) as HTMLInputElement;
    input.value = "Nombre";

    moved();
    press(input, "Enter");
    await settle();

    expect(core.edit).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("tableLost"));
  });

  it("asks no change from a list answered once the page moved on", async () => {
    const { held: room, moved, lost } = made();
    room.onTable(asked("cell", 0, 1));
    await settle();

    moved();
    fireEvent.click(screen.getByText("Video"));
    await settle();

    expect(core.edit).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("tableLost"));
  });

  it("opens no cell on the next document when its words come back after the move", async () => {
    const { held: room, page, moved } = made();
    const words = held<{ heads: string[]; rows: string[][] }>();
    core.written.mockReturnValue(words.asked);
    room.onTable(asked("cell", 1, 0));

    moved();
    words.said.answer({ heads: ["Servicio", "Tipo", "Precio"], rows: [["Spotify"], ["Netflix"]] });
    await settle();

    expect(field(page)).toBeNull();
  });

  it("lets the keys held for the next cell go when that cell opens after the move", async () => {
    const { held: room, page, moved } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const words = held<{ heads: string[]; rows: string[][] }>();
    core.written.mockReturnValueOnce(words.asked);
    press(field(page), "Enter");
    await settle();

    moved();
    words.said.answer({ heads: ["Servicio", "Tipo", "Precio"], rows: [["Spotify"], ["Netflix"]] });
    await settle();
    const key = new KeyboardEvent("keydown", { key: "x", bubbles: true, cancelable: true });
    page.body.dispatchEvent(key);

    expect(key.defaultPrevented).toBe(false);
  });
});

describe("a table's change answered after its page gave way to another document", () => {
  it("writes nothing, opens no next cell there, and says it came too late", async () => {
    const { held: room, page, moved, whole, lost, turn } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValue(answer.asked);
    const input = field(page) as HTMLInputElement;
    input.value = "Tidal";
    press(input, "Enter");
    await settle();

    moved();
    answer.said.answer("next body");
    await settle();
    turn(2);
    await settle();

    expect(whole).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("tableLost"));
    expect(field(page)).toBeNull();
  });
});

describe("a table's change refused after its page gave way to another document", () => {
  it("opens no cell clicked meanwhile on the next document", async () => {
    const { held: room, page, moved } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal: { say: (why: string) => void } = { say: () => {} };
    core.edit.mockReturnValue(
      new Promise((_, refused) => {
        refusal.say = refused;
      }),
    );
    const input = field(page) as HTMLInputElement;
    input.value = "Tidal";
    press(input, "Enter");
    await settle();
    room.onTable(asked("cell", 1, 0));

    moved();
    refusal.say("moved");
    await settle();

    expect(field(page)).toBeNull();
  });
});

describe("a table's menu answered after its page gave way to another document", () => {
  it("asks no change of the next document, and says the choice came too late", async () => {
    const { held: room, moved, lost } = made();
    room.onTable(asked("row", 0, -1));
    await settle();

    moved();
    fireEvent.click(screen.getByRole("menuitem", { name: t("tableRowCopy") }));
    await settle();

    expect(core.edit).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("choiceLost"));
  });
});

describe("a table's change for a page that is no longer shown, where the window keeps words aside", () => {
  const keeping = () => {
    const room = made();
    room.held.kept = [];
    return room;
  };

  it("keeps the change the core answered aside for its own document, and says so", async () => {
    const { held: room, page, moved, whole, lost } = keeping();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValue(answer.asked);
    const input = field(page) as HTMLInputElement;
    input.value = "Tidal";
    press(input, "Enter");
    await settle();

    moved();
    answer.said.answer("next body");
    await settle();

    expect(whole).not.toHaveBeenCalled();
    expect(room.kept).toEqual(["next body"]);
    expect(lost).toHaveBeenCalledWith(t("tableKept"));
  });

  it("works out a cell still taking its form against the text it was typed on, and keeps it aside", async () => {
    const { held: room, page, moved, lost } = keeping();
    room.onTable(asked("cell", 1, 2));
    await settle();
    const form = held<string>();
    core.canon.mockReturnValue(form.asked);
    const input = field(page) as HTMLInputElement;
    input.value = "10990";
    press(input, "Enter");

    moved();
    form.said.answer("10990");
    await settle();

    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 120], {
      kind: "cell",
      row: 1,
      column: 2,
      said: "10990",
    });
    expect(room.kept).toEqual(["next body"]);
    expect(lost).toHaveBeenCalledWith(t("tableKept"));
  });

  it("keeps a title left after the move aside too", async () => {
    const { held: room, page, moved, lost } = keeping();
    room.onTable(asked("cell", -1, 0));
    await settle();
    const input = field(page) as HTMLInputElement;
    input.value = "Nombre";

    moved();
    press(input, "Enter");
    await settle();

    expect(core.edit).toHaveBeenCalledWith(BODY, [0, 120], {
      kind: "rename",
      column: 0,
      said: "Nombre",
    });
    expect(room.kept).toEqual(["next body"]);
    expect(lost).toHaveBeenCalledWith(t("tableKept"));
  });
});

describe("a cell clicked on the next document while the last one's change was on its way", () => {
  it("opens once that change comes back too late for its own page", async () => {
    const { held: room, page, moved } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValue(answer.asked);
    const input = field(page) as HTMLInputElement;
    input.value = "Tidal";
    press(input, "Enter");
    await settle();

    moved();
    room.onTable(asked("cell", 1, 0));
    answer.said.answer("next body");
    await settle();

    expect(field(page)?.value).toBe("Netflix");
  });
});

const celled = (body: string, row: number, column: number, said: string) => {
  const lines = body.split("\n");
  const at = lines.findIndex((one) => one.startsWith("| Servicio")) + 2 + row;
  const cells = lines[at].split("|");
  cells[column + 1] = ` ${said} `;
  lines[at] = cells.join("|");
  return lines.join("\n");
};

const SWAPPED = BODY.replace(
  "| Spotify | Música | 6490 |\n| Netflix | Video | 9990 |",
  "| Netflix | Video | 9990 |\n| Spotify | Música | 6490 |",
);

const refusing = <T,>(promised: { mockReturnValueOnce: (one: Promise<T>) => unknown }) => {
  const refusal: { say: (why: string) => void } = { say: () => {} };
  promised.mockReturnValueOnce(
    new Promise<T>((_, refused) => {
      refusal.say = refused;
    }),
  );
  return refusal;
};

const leftBy = (page: Document, words: string, how: "blur" | KeyboardEventInit) => {
  const input = field(page) as HTMLInputElement;
  input.value = words;
  if (how === "blur") input.dispatchEvent(new FocusEvent("blur"));
  else input.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, ...how }));
};

const typed = (page: Document, keys: string) => {
  for (const key of keys) press(page.body, key);
};

const caught = (page: Document) => {
  const key = new KeyboardEvent("keydown", { key: "z", bubbles: true, cancelable: true });
  page.body.dispatchEvent(key);
  return key.defaultPrevented;
};

const star = (row: number, column: number, to: number): TableWent => ({
  ...asked("star", row, column),
  to,
});

describe("a table's change answered after the same page changed meanwhile", () => {
  beforeEach(() => {
    core.edit.mockImplementation((body, _at, ...edits) =>
      Promise.resolve(
        (edits as { row: number; column: number; said: string }[]).reduce(
          (now, one) => celled(now, one.row, one.column, one.said),
          body,
        ),
      ),
    );
  });

  it("lays the change over words written under the table", async () => {
    const { held: room, page, whole, lost } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", "blur");
    await settle();

    room.text.current = `${BODY}\nNotas de hoy\n`;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(`${celled(BODY, 0, 0, "Tidal")}\nNotas de hoy\n`);
    expect(lost).not.toHaveBeenCalled();
  });

  it("lays the change under words written above the table", async () => {
    const { held: room, page, whole } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", "blur");
    await settle();

    room.text.current = `Hola\n\n${BODY}`;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(`Hola\n\n${celled(BODY, 0, 0, "Tidal")}`);
  });

  it("keeps both of two changes on their way at once in different rows", async () => {
    const { held: room, page, whole } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", "blur");
    await settle();

    room.onTable(star(1, 1, 3));
    await settle();
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(celled(celled(BODY, 0, 0, "Tidal"), 1, 1, "3"));
  });

  it("gives a cell the last of two changes asked of it at once", async () => {
    const { held: room, whole, lost } = made();
    const first = held<string>();
    const second = held<string>();
    core.edit.mockReturnValueOnce(first.asked).mockReturnValueOnce(second.asked);
    room.onTable(star(1, 1, 3));
    room.onTable(star(1, 1, 4));
    await settle();

    first.said.answer(celled(BODY, 1, 1, "3"));
    await settle();
    second.said.answer(celled(BODY, 1, 1, "4"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(celled(BODY, 1, 1, "4"));
    expect(lost).not.toHaveBeenCalled();
  });

  it("sets the change aside when asking it again of the text as it is now is refused", async () => {
    const { held: room, whole, lost } = made();
    room.kept = [];
    const first = held<string>();
    const second = held<string>();
    core.edit
      .mockReturnValueOnce(first.asked)
      .mockReturnValueOnce(second.asked)
      .mockRejectedValueOnce("moved");
    room.onTable(star(1, 1, 3));
    room.onTable(star(1, 1, 4));
    await settle();

    first.said.answer(celled(BODY, 1, 1, "3"));
    await settle();
    second.said.answer(celled(BODY, 1, 1, "4"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(celled(BODY, 1, 1, "3"));
    expect(room.kept).toEqual([celled(BODY, 1, 1, "4")]);
    expect(lost).toHaveBeenLastCalledWith(t("tableKeptOver"));
  });

  it("begins no line under the table on Enter from its last row once words came above it", async () => {
    const { held: room, page, whole, passed } = made();
    room.onTable(asked("cell", 1, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", { key: "Enter" });
    await settle();

    room.text.current = `Hola\n\n${BODY}`;
    answer.said.answer(celled(BODY, 1, 0, "Tidal"));
    await settle();

    expect(whole).toHaveBeenLastCalledWith(`Hola\n\n${celled(BODY, 1, 0, "Tidal")}`);
    expect(passed).not.toHaveBeenCalled();
  });

  it("keeps what changed meanwhile and sets the change aside when both touch the same rows", async () => {
    const { held: room, page, whole, lost } = made();
    room.kept = [];
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", "blur");
    await settle();

    room.text.current = SWAPPED;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(whole).not.toHaveBeenCalled();
    expect(room.text.current).toBe(SWAPPED);
    expect(room.kept).toEqual([celled(BODY, 0, 0, "Tidal")]);
    expect(lost).toHaveBeenCalledWith(t("tableKeptOver"));
  });

  it("says the change was not made where nothing keeps words aside", async () => {
    const { held: room, page, whole, lost } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", "blur");
    await settle();

    room.text.current = SWAPPED;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(whole).not.toHaveBeenCalled();
    expect(lost).toHaveBeenCalledWith(t("tableNotLaid"));
  });

  it("writes the keys held for the next cell under the table when the change is set aside", async () => {
    const { held: room, page, passed } = made();
    room.kept = [];
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", { key: "Enter" });
    await settle();
    typed(page, "ab");

    room.text.current = SWAPPED;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(caught(page)).toBe(false);
    expect(field(page)).toBeNull();
    expect(passed).toHaveBeenCalledWith(expect.any(Number), [0, 120], "ab");
    expect(room.told).toEqual([t("tableKeysUnder"), t("tableKeptOver")]);
  });

  it("waits for the page to be drawn anew before writing them under the table", async () => {
    const { held: room, page, passed, turn } = made();
    room.kept = [];
    room.onTable(asked("cell", 0, 0));
    await settle();
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    leftBy(page, "Tidal", { key: "Enter" });
    await settle();
    typed(page, "ab");

    room.text.current = SWAPPED;
    room.behind = BODY;
    answer.said.answer(celled(BODY, 0, 0, "Tidal"));
    await settle();

    expect(caught(page)).toBe(false);
    expect(passed).not.toHaveBeenCalled();
    room.behind = null;
    turn(2);
    await settle();
    expect(passed).toHaveBeenCalledWith(expect.any(Number), [0, 120], "ab");
  });
});

describe("a table's change for a page now held by another sheet that changed meanwhile", () => {
  it("lands there over the words written meanwhile", async () => {
    const { held: room, moved, lost } = made();
    const there = `${BODY}\nNotas de hoy\n`;
    const landed: string[] = [];
    room.seat.now = {
      current: (next) => {
        const now = next(there);
        if (now !== null) landed.push(now);
        return now !== null;
      },
    };
    const answer = held<string>();
    core.edit.mockReturnValueOnce(answer.asked);
    room.onTable(star(1, 1, 3));
    await settle();

    moved();
    answer.said.answer(celled(BODY, 1, 1, "3"));
    await settle();

    expect(landed).toEqual([`${celled(BODY, 1, 1, "3")}\nNotas de hoy\n`]);
    expect(lost).not.toHaveBeenCalled();
  });
});

describe("keys held for the next cell when a step on the way fails", () => {
  it("lets them go at once and writes them under the table when that cell cannot be read", async () => {
    const { held: room, page, lost, passed } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal = refusing(core.written);
    press(field(page), "Enter");
    await settle();
    typed(page, "ab");

    refusal.say("moved");
    await settle();

    expect(caught(page)).toBe(false);
    expect(passed).toHaveBeenCalledWith(expect.any(Number), [0, 120], "ab");
    expect(lost).toHaveBeenCalledWith(t("tableKeysUnder"));
  });

  it("names them in its notice when no line could be begun under the table", async () => {
    const { held: room, page, lost, passed } = made();
    passed.mockReturnValue(false);
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal = refusing(core.written);
    press(field(page), "Enter");
    await settle();
    typed(page, "ab");

    refusal.say("moved");
    await settle();

    expect(caught(page)).toBe(false);
    expect(lost).toHaveBeenCalledWith(fill("tableKeysLost", "ab"));
    expect(lost).not.toHaveBeenCalledWith(t("tableKeysUnder"));
  });

  it("lets the catcher go and says why when nothing was typed yet for that cell", async () => {
    const { held: room, page, lost, passed, fail } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal = refusing(core.written);
    press(field(page), "Enter");
    await settle();

    refusal.say("moved");
    await settle();

    expect(caught(page)).toBe(false);
    expect(passed).not.toHaveBeenCalled();
    expect(lost).not.toHaveBeenCalled();
    expect(fail).toHaveBeenCalledTimes(1);
  });

  it("puts them in the next cell when the change of the one left is refused", async () => {
    const { held: room, page, passed } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal = refusing(core.edit);
    leftBy(page, "Tidal", { key: "Enter" });
    await settle();
    typed(page, "ab");

    refusal.say("beyond");
    await settle();

    expect(field(page)?.value).toBe("ab");
    expect(field(page)?.closest("td")?.textContent).toContain("Netflix");
    expect(caught(page)).toBe(false);
    expect(passed).not.toHaveBeenCalled();
  });

  it("puts them back in the cell left when moving its row is refused", async () => {
    const { held: room, page } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const refusal = refusing(core.edit);
    leftBy(page, "Tidal", { key: "ArrowDown", altKey: true });
    await settle();
    typed(page, "ab");

    refusal.say("beyond");
    await settle();

    expect(field(page)?.value).toBe("ab");
    expect(field(page)?.closest("td")?.textContent).toContain("Spotify");
  });

  it("writes them under the table when the row past the last cell is refused", async () => {
    const { held: room, page, passed } = made();
    room.onTable(asked("cell", 1, 2));
    await settle();
    const refusal = refusing(core.edit);
    press(field(page), "Tab");
    await settle();
    typed(page, "ab");

    refusal.say("beyond");
    await settle();

    expect(caught(page)).toBe(false);
    expect(field(page)).toBeNull();
    expect(passed).toHaveBeenCalledWith(expect.any(Number), [0, 120], "ab");
    expect(room.told[0]).toBe(t("tableKeysUnder"));
    expect(room.told[room.told.length - 1]).toBeInstanceOf(Error);
  });
});

const BOTH_ROWS = {
  heads: ["Servicio", "Tipo", "Precio"],
  rows: [
    ["Spotify", "Música", "6490"],
    ["Netflix", "Video", "9990"],
  ],
};

describe("keys held for a next cell written by picking from a list", () => {
  it("lets them go when its list opens and puts them in the list's own field", async () => {
    const { held: room, page } = made();
    room.onTable(asked("cell", 0, 0));
    await settle();
    const words = held<{ heads: string[]; rows: string[][] }>();
    core.written.mockReturnValueOnce(words.asked);
    press(field(page), "Tab");
    await settle();
    typed(page, "Jazz");

    words.said.answer(BOTH_ROWS);
    await settle();

    expect(caught(page)).toBe(false);
    expect(screen.getByRole("textbox", { name: t("tableOptionNew") })).toHaveProperty(
      "value",
      "Jazz",
    );
  });

  it("puts them in the search of a list of documents", async () => {
    const { held: room, page, passed } = made();
    const head = page.querySelectorAll<HTMLElement>("thead th")[1];
    head.dataset.type = "link";
    room.onTable(asked("cell", 0, 0));
    await settle();
    const words = held<{ heads: string[]; rows: string[][] }>();
    core.written.mockReturnValueOnce(words.asked);
    press(field(page), "Tab");
    await settle();
    typed(page, "ab");

    words.said.answer(BOTH_ROWS);
    await settle();

    expect(caught(page)).toBe(false);
    expect(screen.getByRole("combobox", { name: t("pickADocToLink") })).toHaveProperty(
      "value",
      "ab",
    );
    expect(passed).not.toHaveBeenCalled();
  });
});
