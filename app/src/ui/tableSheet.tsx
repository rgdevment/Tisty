import { type ReactNode, useEffect, useRef, useState } from "react";
import { charsAt, tablesIn, today } from "tisty-editor/read";
import { fill, t } from "../locales";
import {
  refused,
  type TableEdit,
  tableCanon,
  tableEdit,
  tableOrder,
  tableWritten,
} from "../tables";
import { cellBar } from "./cellBar";
import Menu, { type Choice } from "./Menu";
import { type Seated, seatedNow } from "./pageLate";
import { placeOf } from "./pagePlaces";
import type { TableWent } from "./paging";
import { type Mark, type Peeled, peeled, toggled } from "./peeled";
import { seatedChoices } from "./seated";
import TableEditor, { type Picked } from "./TableEditor";
import { columnChoices, wordsOf } from "./tableChoices";
import {
  beside,
  cellOf,
  type Filtering,
  headsNamed,
  type Leaving,
  nthOf,
  rowsCount,
  rowsShown,
  tablePlace,
  typing,
  type Viewing,
  viewed,
} from "./tabled";
import { endAfter, laidOver, rowAskedAgain } from "./tableMeanwhile";

type Ways = {
  /** The text as it is now, and as the page on screen was drawn from. */
  readonly latest: { readonly current: string };
  readonly drawn: { readonly current: string };
  readonly page: () => Document | null;
  readonly frame: () => DOMRect | undefined;
  readonly whole: (next: string) => void;
  /** Enter on the last row leaves the table: a line begins under it, at this place in the text. */
  readonly passed?: (end: number, table: readonly [number, number], typed?: string) => boolean;
  readonly writing: () => boolean;
  readonly leave: () => void;
  readonly markdown: (at: [number, number], box: HTMLElement) => void;
  readonly fail: (problem: unknown) => void;
  readonly seat: () => Seated["seat"];
  readonly lost: (notice: string) => void;
  readonly aside: () => ((words: string) => void) | null;
  /** Counts each new drawing of the page. */
  readonly turned: number;
};

// The types written by picking from a list or a calendar rather than typing in the cell.
const PICKED = new Set(["date", "select", "tags", "link"]);

type Place = { readonly nth: number; readonly row: number; readonly column: number };

type Started = Seated & {
  readonly was: string;
  readonly aside: ((words: string) => void) | null;
};

type Opened = Started & { readonly at: [number, number] };

// A column's options are the values its cells already hold, a tag list split into its tags.
const optionsIn = (page: Document, nth: number, column: number, tags: boolean): string[] => {
  const seen = new Map<string, string>();
  for (const row of rowsShown(page, nth)) {
    const said = cellOf(page, nth, row, column)?.textContent?.trim() ?? "";
    for (const one of tags ? said.split(",") : [said]) {
      const kept = one.trim();
      if (kept && !seen.has(kept.toLowerCase())) seen.set(kept.toLowerCase(), kept);
    }
  }
  return [...seen.values()];
};

// Columns that move in the file move what is shown with them; a column gone takes its sort and filters along.
const moved = (view: Viewing, to: (column: number) => number | null): Viewing => {
  const column = view.sort ? to(view.sort.column) : null;
  return {
    sort: view.sort && column !== null ? { ...view.sort, column, order: [] } : undefined,
    filters: view.filters.flatMap((one) => {
      const now = to(one.column);
      return now === null ? [] : [{ ...one, column: now }];
    }),
  };
};

const unlanded = (then: Place, edits: readonly TableEdit[]): Place | null => {
  if (edits.some((one) => one.kind === "rowAdd")) return null;
  const move = edits.find((one) => one.kind === "rowMove");
  return move?.kind === "rowMove" ? { ...then, row: move.from } : then;
};

/** What the page's tables do when they are written in place: cells, rows, columns, and how each is shown. */
export const useTables = (
  ways: Ways,
): {
  onTable: (asked: TableWent) => void;
  menu: ReactNode;
  freshAt: (byte: number) => void;
  busy: () => boolean;
  whenIdle: (then: () => void) => void;
} => {
  const views = useRef(new Map<number, Viewing>());
  const counted = useRef(-1);
  const pending = useRef<Place | null>(null);
  // A table just made in the text, by the byte it starts at, waits for the page that draws it.
  const born = useRef<number | null>(null);
  // A cell clicked while the one left behind is still being saved opens once that save is drawn.
  const saving = useRef(0);
  const idle = useRef<(() => void)[]>([]);
  const saved = () => {
    saving.current -= 1;
    if (saving.current > 0) return;
    const waiting = idle.current;
    idle.current = [];
    for (const one of waiting) one();
  };
  const wanted = useRef<{ place: Place; seat: Seated["seat"] } | null>(null);
  // Keys pressed while the next cell is saved and opened are held for it instead of falling on the page.
  const ahead = useRef<{ said: string; stop: () => void; seat: Seated["seat"] } | null>(null);
  const leftover = useRef("");
  const backed = useRef<{ words: string; nth: number } | null>(null);
  const [picking, setPicking] = useState<{
    kind: Picked;
    at: { x: number; y: number; above: number };
    was: string;
    shown: string;
    options: string[];
    typed: string;
    place: Place;
    here: Opened;
  } | null>(null);
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    top?: number;
    right?: number;
    label: string;
    choices: Choice[];
  } | null>(null);
  const said = useRef(ways);
  said.current = ways;

  // A click on a page still on screen while the next one is drawn names bytes of an older text.
  const fresh = () => said.current.drawn.current === said.current.latest.current;

  const seatedHere = () => seatedNow(() => said.current.seat());

  const started = (): Started => ({
    ...seatedHere(),
    was: said.current.latest.current,
    aside: said.current.aside(),
  });

  const letGo = (here: Seated) => {
    if (ahead.current?.seat === here.seat) ahead.current.stop();
    const clicked = wanted.current;
    wanted.current = null;
    if (clicked && clicked.seat !== here.seat) opened(clicked.place);
  };

  const keptAway = (here: Started, next: string) => {
    if (here.seat?.current?.((now) => laidOver(here.was, next, now))) return;
    if (!here.aside) return said.current.lost(t("tableLost"));
    here.aside(next);
    said.current.lost(t("tableKept"));
  };

  const heldWords = (): string => {
    const held = ahead.current;
    held?.stop();
    return held ? held.said.replace(/[\t\n]+/g, " ").trim() : "";
  };

  const laidUnder = () => {
    const back = backed.current;
    if (!back || !fresh()) return;
    backed.current = null;
    const page = said.current.page();
    const at = page && tablePlace(page, back.nth);
    const end = at ? charsAt(said.current.latest.current, at[1]) : 0;
    if (at && said.current.passed?.(end, at, back.words))
      return said.current.lost(t("tableKeysUnder"));
    said.current.lost(fill("tableKeysLost", back.words));
  };

  const handedBack = (nth: number): boolean => {
    const words = heldWords();
    if (!words) return false;
    backed.current = { words, nth };
    laidUnder();
    return true;
  };

  const goneOn = (open: Place | null, toward: Place | null) => {
    const next = wanted.current?.place ?? open;
    wanted.current = null;
    if (next) opened(next);
    else handedBack(toward?.nth ?? -1);
  };

  const keptOver = (here: Started, next: string, toward: Place | null) => {
    goneOn(null, toward);
    if (!here.aside) return said.current.lost(t("tableNotLaid"));
    here.aside(next);
    said.current.lost(t("tableKeptOver"));
  };

  const movedOff = (here: Opened, edits: TableEdit[]) => {
    letGo(here);
    if (!here.seat?.current && !here.aside) return said.current.lost(t("tableLost"));
    tableEdit(here.was, here.at, ...edits).then(
      (next) => keptAway(here, next),
      () => said.current.lost(t("tableLost")),
    );
  };

  const edited = (
    at: [number, number],
    edits: TableEdit[],
    then: Place | null = null,
    settled?: () => void,
    passing = false,
  ) => {
    if (!fresh()) return;
    saving.current += 1;
    const here = started();
    const missed = (was: string, next: string) => {
      if (here.still()) return keptOver(here, next, then);
      letGo(here);
      keptAway({ ...here, was }, next);
    };
    const asked = (was: string, table: [number, number]): Promise<void> =>
      tableEdit(was, table, ...edits).then((next) => {
        if (!here.still()) return missed(was, next);
        const now = said.current.latest.current;
        const again = now === next ? null : rowAskedAgain(was, now, table);
        if (again) return asked(now, again).catch(() => missed(was, next));
        const laid = laidOver(was, next, now);
        if (laid === null) return missed(was, next);
        settled?.();
        pending.current = then ?? wanted.current?.place ?? null;
        wanted.current = null;
        said.current.whole(laid);
        if (passing && laid === next) said.current.passed?.(endAfter(was, next, table), table);
      });
    asked(here.was, at)
      .catch((why) => {
        if (here.still()) goneOn(then && unlanded(then, edits), then);
        else letGo(here);
        said.current.fail(refused(why));
      })
      .finally(saved);
  };

  const remapped = (nth: number, to: (column: number) => number | null) => {
    const was = views.current.get(nth);
    if (was) views.current.set(nth, moved(was, to));
  };

  // A new row is opened to be written, so no filter may hide it.
  const unfiltered = (nth: number) => {
    const was = views.current.get(nth);
    if (was) views.current.set(nth, { ...was, filters: [] });
  };

  const shown = (nth: number) => {
    const page = said.current.page();
    const at = page && tablePlace(page, nth);
    if (!page || !at) return;
    const view = views.current.get(nth);
    const sort = view?.sort;
    // An order already asked for this drawing is laid again as it is; only a new sort or drawing asks anew.
    if (!sort || sort.order.length > 0) {
      viewed(page, nth, view, today());
      return;
    }
    tableOrder(said.current.latest.current, at, sort.column, sort.down)
      .then((order) => {
        const now = views.current.get(nth);
        if (now?.sort) views.current.set(nth, { ...now, sort: { ...now.sort, order } });
        viewed(page, nth, views.current.get(nth), today());
      })
      .catch(() => {});
  };

  const viewing = (nth: number, change: (was: Viewing) => Viewing) => {
    views.current.set(nth, change(views.current.get(nth) ?? { filters: [] }));
    shown(nth);
  };

  // What is typed in a typed column is written in that type's own form; a plain column keeps it as typed.
  const canonical = (page: Document, place: Place, typed: string): Promise<string> => {
    const head = cellOf(page, place.nth, -1, place.column);
    const type = head?.dataset.type;
    if (!head || !type) return Promise.resolve(typed);
    const words = wordsOf(type, head.dataset.currency ?? "", head.dataset.total ?? "");
    return tableCanon(words || "text", typed).catch(() => typed);
  };

  const written = (place: Place, typed: string, here: Opened) => {
    const page = said.current.page();
    if (!page) return;
    const cells = (said: string): TableEdit[] => [
      { kind: "cell", row: place.row, column: place.column, said },
    ];
    if (!here.still()) return movedOff(here, cells(typed));
    canonical(page, place, typed)
      .then((said) => {
        if (!here.still()) return movedOff(here, cells(said));
        const at = tablePlace(page, place.nth);
        if (at) edited(at, cells(said));
      })
      .catch((why) => said.current.fail(refused(why)));
  };

  const finished = (
    place: Place,
    cell: Peeled,
    now: string,
    how: Leaving,
    marks: readonly Mark[],
    here: Opened,
  ) => {
    if (how === "cancel") return;
    const restyled = JSON.stringify(marks) !== JSON.stringify(cell.marks);
    const changed = restyled || now.trim() !== cell.words.trim();
    const { row, column } = place;
    const words = (said: string): TableEdit[] =>
      row < 0
        ? [{ kind: "rename", column, said: cell.wrap(said, marks) }]
        : [{ kind: "cell", row, column, said: cell.wrap(said, marks) }];
    if (!here.still()) return changed ? movedOff(here, words(now)) : letGo(here);
    const page = said.current.page();
    const at = page && tablePlace(page, place.nth);
    if (!page || !at || !fresh()) return;
    if (how !== "blur") awaited(page);
    if (row < 0) {
      onward(place, changed ? words(now) : [], how);
      return;
    }
    if (!changed) {
      onward(place, [], how);
      return;
    }
    // The save starts once the value is in its form, and a cell clicked meanwhile waits for it.
    saving.current += 1;
    // The words take the column's form first; the marks around the cell then wrap that form.
    canonical(page, place, now)
      .then((said) =>
        here.still() ? onward(place, words(said), how) : movedOff(here, words(said)),
      )
      .catch((why) => {
        if (here.still()) handedBack(place.nth);
        said.current.fail(refused(why));
      })
      .finally(saved);
  };

  const awaited = (page: Document) => {
    if (ahead.current) return;
    const held = { said: leftover.current, stop: () => {}, seat: said.current.seat() };
    leftover.current = "";
    const caught = (event: KeyboardEvent) => {
      if ((event.target as Element | null)?.closest?.(".table-input")) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      const moved =
        event.key === "Tab" ? (event.shiftKey ? "" : "\t") : event.key === "Enter" ? "\n" : null;
      if (moved === null && event.key.length !== 1) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      held.said += moved ?? event.key;
    };
    const gone = setTimeout(() => held.stop(), 3000);
    held.stop = () => {
      clearTimeout(gone);
      page.removeEventListener("keydown", caught, true);
      if (ahead.current === held) ahead.current = null;
    };
    page.addEventListener("keydown", caught, true);
    ahead.current = held;
  };

  const onward = (place: Place, cell: TableEdit[], how: Leaving) => {
    const page = said.current.page();
    const at = page && tablePlace(page, place.nth);
    if (!page || !at) return;
    if (how !== "blur" && how !== "cancel") awaited(page);
    const { nth, row, column } = place;
    if ((how === "up" || how === "down") && row < 0) {
      ahead.current?.stop();
      if (cell.length) edited(at, cell);
      return;
    }
    if (how === "up" || how === "down") {
      const to = row + (how === "up" ? -1 : 1);
      if (to < 0 || to >= rowsCount(page, nth)) {
        if (cell.length) edited(at, cell, place);
        return;
      }
      viewing(nth, (one) => ({ ...one, sort: undefined }));
      edited(at, [...cell, { kind: "rowMove", from: row, to }], { nth, row: to, column });
      return;
    }
    const next = beside(rowsShown(page, nth), headsNamed(page, nth).length, row, column, how);
    if (next === "new") {
      const rows = rowsCount(page, nth);
      edited(at, [...cell, { kind: "rowAdd", at: rows }], { nth, row: rows, column: 0 }, () =>
        unfiltered(nth),
      );
      return;
    }
    const then = next && how !== "blur" ? { nth, ...next } : null;
    if (!then) ahead.current?.stop();
    const passing = !then && how === "enter";
    if (cell.length) edited(at, cell, then, undefined, passing);
    else if (then) opened(then);
    else if (passing) said.current.passed?.(charsAt(said.current.latest.current, at[1]), at);
  };

  const opened = (place: Place) => {
    const page = said.current.page();
    const at = page && tablePlace(page, place.nth);
    const holder = page && cellOf(page, place.nth, place.row, place.column);
    if (!page || !at || !holder) {
      handedBack(place.nth);
      return;
    }
    if (!fresh()) {
      pending.current = place;
      return;
    }
    const type = cellOf(page, place.nth, -1, place.column)?.dataset.type ?? "";
    const here: Opened = { ...started(), at };
    tableWritten(said.current.latest.current, at)
      .then((written) => {
        if (!here.still()) return letGo(here);
        const was =
          place.row < 0 ? written.heads[place.column] : written.rows[place.row]?.[place.column];
        if (place.row >= 0 && PICKED.has(type)) {
          const box = said.current.frame();
          const cell = holder.getBoundingClientRect();
          setPicking({
            kind: type as Picked,
            at: {
              x: (box?.left ?? 0) + cell.left,
              y: (box?.top ?? 0) + cell.bottom,
              above: (box?.top ?? 0) + cell.top,
            },
            was: was ?? "",
            shown: holder.textContent?.trim() ?? "",
            options: optionsIn(page, place.nth, place.column, type === "tags"),
            typed: heldWords(),
            place,
            here,
          });
          return;
        }
        const name = headsNamed(page, place.nth)[place.column] ?? "";
        const label = place.row < 0 ? t("tableHead") : fill("tableCell", name);
        // The field holds the words; the marks wrapping the whole cell come back around them on the way out.
        const cell = peeled(was ?? "");
        let marks = cell.marks;
        let unbar = () => {};
        const held = ahead.current?.said ?? "";
        ahead.current?.stop();
        const step = held.search(/[\t\n]/);
        const typed = step < 0 ? held : held.slice(0, step);
        const input = typing(holder, typed || cell.words, label, (now, how) => {
          unbar();
          finished(place, cell, now, how, marks, here);
        });
        if (typed) input.setSelectionRange(typed.length, typed.length);
        if (step >= 0) {
          leftover.current = held.slice(step + 1);
          const key = held[step] === "\t" ? "Tab" : "Enter";
          input.dispatchEvent(
            new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }),
          );
          return;
        }
        const box = tablesIn(page)[place.nth];
        if (box)
          unbar = cellBar(
            box,
            input,
            () => marks,
            (open, close) => {
              marks = toggled(marks, open, close);
            },
          );
      })
      .catch((why) => {
        if (!here.still()) letGo(here);
        else if (handedBack(place.nth)) return;
        said.current.fail(refused(why));
      });
  };

  // Each new drawing of the page comes plain: what is shown on screen is laid on it again.
  const relaid = () => {
    const page = said.current.page();
    if (!page) return;
    // A table added or taken away elsewhere moves every table after it, so what each showed starts over.
    const many = tablesIn(page).length;
    if (many !== counted.current) views.current.clear();
    counted.current = many;
    for (const [nth, view] of views.current) {
      if (view.sort) views.current.set(nth, { ...view, sort: { ...view.sort, order: [] } });
      shown(nth);
    }
    const made = born.current;
    const nth = made === null ? -1 : tablesIn(page).findIndex((one) => placeOf(one)?.[0] === made);
    if (nth >= 0) {
      born.current = null;
      pending.current = { nth, row: -1, column: 0 };
    }
    const next = pending.current;
    pending.current = null;
    if (next) opened(next);
    laidUnder();
  };
  const relay = useRef(relaid);
  relay.current = relaid;
  const turned = ways.turned;
  useEffect(() => {
    if (turned > 0) relay.current();
  }, [turned]);

  const rowChoices = (asked: TableWent, nth: number): Choice[] => {
    const page = said.current.page();
    const rows = page ? rowsCount(page, nth) : 0;
    const moved = (to: number) => () => {
      viewing(nth, (one) => ({ ...one, sort: undefined }));
      edited(asked.at, [{ kind: "rowMove", from: asked.row, to }]);
    };
    return [
      {
        key: "up",
        label: t("tableRowUp"),
        icon: "↑",
        hint: "Alt ↑",
        off: asked.row <= 0,
        onPick: moved(asked.row - 1),
      },
      {
        key: "down",
        label: t("tableRowDown"),
        icon: "↓",
        hint: "Alt ↓",
        off: asked.row >= rows - 1,
        onPick: moved(asked.row + 1),
      },
      {
        key: "copy",
        label: t("tableRowCopy"),
        icon: "⧉",
        onPick: () => edited(asked.at, [{ kind: "rowCopy", row: asked.row }]),
      },
      {
        key: "remove",
        label: t("tableRowRemove"),
        icon: "✕",
        danger: true,
        apart: true,
        onPick: () => edited(asked.at, [{ kind: "rowRemove", row: asked.row }]),
      },
    ];
  };

  const filtering = (nth: number, column: number, name: string) => {
    const page = said.current.page();
    const bar = page && tablesIn(page)[nth]?.querySelector<HTMLElement>(":scope > .table-bar");
    if (!bar) return;
    bar.hidden = false;
    typing(bar, "", t("tableFilterHas"), (now, how) => {
      const kept = now.trim();
      if (how === "cancel" || !kept) {
        shown(nth);
        return;
      }
      const one: Filtering = {
        column,
        kind: "has",
        said: kept,
        label: fill("tableFilterHasSaid", name, kept),
      };
      viewing(nth, (was) => ({ ...was, filters: [...was.filters, one] }));
    });
  };

  const columnMenu = (asked: TableWent, nth: number): Choice[] => {
    const page = said.current.page();
    const box = page && tablesIn(page)[nth];
    const head = box?.querySelectorAll<HTMLElement>(":scope > table > thead th")[asked.column];
    if (!page || !box || !head) return [];
    const name = headsNamed(page, nth)[asked.column] ?? "";
    const type = head.dataset.type ?? "";
    const { column } = asked;
    return columnChoices(
      { name, type, total: head.dataset.total ?? "", currency: head.dataset.currency ?? "" },
      optionsIn(page, nth, column, type === "tags"),
      {
        sort: (down) => sorted(nth, column, down),
        filter: (one) =>
          viewing(nth, (was) => ({ ...was, filters: [...was.filters, { ...one, column }] })),
        asking: () => filtering(nth, column, name),
        // A new type reads its cells anew, so a filter made for the old one goes.
        retype: (words) =>
          edited(asked.at, [{ kind: "retype", column, words }], null, () => {
            const was = views.current.get(nth);
            if (was)
              views.current.set(nth, {
                ...was,
                filters: was.filters.filter((one) => one.column !== column),
              });
          }),
        rename: () => opened({ nth, row: -1, column }),
        insert: (right) => {
          const at = column + (right ? 1 : 0);
          edited(asked.at, [{ kind: "columnAdd", at }], null, () =>
            remapped(nth, (one) => (one >= at ? one + 1 : one)),
          );
        },
        remove: () =>
          edited(asked.at, [{ kind: "columnRemove", column }], null, () =>
            remapped(nth, (one) => (one === column ? null : one > column ? one - 1 : one)),
          ),
        markdown: () => said.current.markdown(asked.at, box),
      },
    );
  };

  const sorted = (nth: number, column: number, down: boolean) =>
    viewing(nth, (was) => ({ ...was, sort: { column, down, order: [] } }));

  const onTable = (asked: TableWent) => {
    const now = said.current;
    const page = now.page();
    const nth = page ? nthOf(page, asked.at) : -1;
    if (nth < 0) return;
    if (now.writing()) {
      now.leave();
      return;
    }
    const box = now.frame();
    const [left, top] = [box?.left ?? 0, box?.top ?? 0];
    const at = {
      x: left + asked.x,
      y: top + asked.y,
      ...(asked.top !== undefined && { top: top + asked.top }),
      ...(asked.right !== undefined && { right: left + asked.right }),
    };
    const name = headsNamed(page as Document, nth)[asked.column] ?? "";
    const here = seatedHere();
    const kept = (choices: Choice[]) =>
      seatedChoices(choices, here.still, () => said.current.lost(t("choiceLost")));
    if (asked.act === "cell") {
      const place = { nth, row: asked.row, column: asked.column };
      if (saving.current > 0) wanted.current = { place, seat: said.current.seat() };
      else opened(place);
    }
    if (asked.act === "star" || asked.act === "toggle") {
      const to = asked.to ?? 0;
      const starred = to ? String(to) : "";
      const ticked = to ? "[x]" : "[ ]";
      const said = asked.act === "star" ? starred : ticked;
      edited(asked.at, [{ kind: "cell", row: asked.row, column: asked.column, said }]);
    }
    if (asked.act === "sort") {
      const was = views.current.get(nth)?.sort;
      sorted(nth, asked.column, was?.column === asked.column && !was.down);
    }
    if (asked.act === "unsort") viewing(nth, (was) => ({ ...was, sort: undefined }));
    if (asked.act === "unfilter")
      viewing(nth, (was) => ({
        ...was,
        filters: was.filters.filter((_, one) => one !== asked.column),
      }));
    if (asked.act === "keep") {
      const sort = views.current.get(nth)?.sort;
      if (!sort || !fresh()) return;
      views.current.set(nth, { ...(views.current.get(nth) ?? { filters: [] }), sort: undefined });
      edited(asked.at, [{ kind: "sort", column: sort.column, down: sort.down }]);
    }
    if (asked.act === "rowAdd") {
      const rows = rowsCount(page as Document, nth);
      edited(asked.at, [{ kind: "rowAdd", at: rows }], { nth, row: rows, column: 0 }, () =>
        unfiltered(nth),
      );
    }
    if (asked.act === "row" && asked.to !== undefined) {
      viewing(nth, (was) => ({ ...was, sort: undefined }));
      edited(asked.at, [{ kind: "rowMove", from: asked.row, to: asked.to }]);
      return;
    }
    if (asked.act === "row")
      setMenu({ ...at, label: t("tableRow"), choices: kept(rowChoices(asked, nth)) });
    if (asked.act === "column")
      setMenu({ ...at, label: fill("tableColumn", name), choices: kept(columnMenu(asked, nth)) });
  };

  const open = () => Boolean(said.current.page()?.querySelector(".table-input"));
  const busy = () => open() || saving.current > 0;
  return {
    onTable,
    freshAt: (byte) => {
      born.current = byte;
    },
    busy,
    // A cell left open is closed first, and what follows waits until its words are saved.
    whenIdle: (then) => {
      said.current.page()?.querySelector(".table-input")?.dispatchEvent(new FocusEvent("blur"));
      if (saving.current > 0) idle.current.push(then);
      else then();
    },
    menu: (
      <>
        {menu && (
          <Menu
            at={{ x: menu.x, y: menu.y, top: menu.top, right: menu.right }}
            label={menu.label}
            choices={menu.choices}
            onClose={() => setMenu(null)}
          />
        )}
        {picking && (
          <TableEditor
            kind={picking.kind}
            at={picking.at}
            was={picking.was}
            shown={picking.shown}
            options={picking.options}
            typed={picking.typed}
            onWrite={(typed) => {
              setPicking(null);
              written(picking.place, typed, picking.here);
            }}
            onClose={() => setPicking(null)}
          />
        )}
      </>
    ),
  };
};
