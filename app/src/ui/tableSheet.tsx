import { type ReactNode, useEffect, useRef, useState } from "react";
import { fill, t } from "../locales";
import { type TableEdit, tableCanon, tableEdit, tableOrder, tableWritten } from "../tables";
import { today } from "./celled";
import Menu, { type Choice } from "./Menu";
import type { TableWent } from "./paging";
import TableEditor, { type Picked } from "./TableEditor";
import { columnChoices, wordsOf } from "./tableChoices";
import {
  beside,
  cellOf,
  type Filtering,
  headsNamed,
  type Leaving,
  nthOf,
  placeOf,
  rowsCount,
  rowsShown,
  tablesIn,
  typing,
  type Viewing,
  viewed,
} from "./tabled";

type Ways = {
  /** The text as it is now, and as the page on screen was drawn from. */
  readonly latest: { readonly current: string };
  readonly drawn: { readonly current: string };
  readonly page: () => Document | null;
  readonly frame: () => DOMRect | undefined;
  readonly whole: (next: string) => void;
  readonly writing: () => boolean;
  readonly leave: () => void;
  readonly markdown: (at: [number, number], box: HTMLElement) => void;
  readonly fail: (problem: unknown) => void;
  /** Counts each new drawing of the page. */
  readonly turned: number;
};

// The types written by picking from a list or a calendar rather than typing in the cell.
const PICKED = new Set(["date", "select", "tags", "link"]);

type Place = { readonly nth: number; readonly row: number; readonly column: number };

const refused = (why: unknown): Error =>
  new Error(
    why === "moved"
      ? t("tableRefusedMoved")
      : why === "lastColumn"
        ? t("tableRefusedLastColumn")
        : t("tableRefused"),
  );

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

/** What the page's tables do when they are written in place: cells, rows, columns, and how each is shown. */
export const useTables = (ways: Ways): { onTable: (asked: TableWent) => void; menu: ReactNode } => {
  const views = useRef(new Map<number, Viewing>());
  const counted = useRef(-1);
  const pending = useRef<Place | null>(null);
  // A cell clicked while the one left behind is still being saved opens once that save is drawn.
  const saving = useRef(0);
  const wanted = useRef<Place | null>(null);
  const [picking, setPicking] = useState<{
    kind: Picked;
    at: { x: number; y: number; above: number };
    was: string;
    shown: string;
    options: string[];
    place: Place;
  } | null>(null);
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    label: string;
    choices: Choice[];
  } | null>(null);
  const said = useRef(ways);
  said.current = ways;

  // A click on a page still on screen while the next one is drawn names bytes of an older text.
  const fresh = () => said.current.drawn.current === said.current.latest.current;

  const edited = (
    at: [number, number],
    edits: TableEdit[],
    then: Place | null = null,
    settled?: () => void,
  ) => {
    if (!fresh()) return;
    saving.current += 1;
    tableEdit(said.current.latest.current, at, ...edits)
      .then((next) => {
        settled?.();
        pending.current = then ?? wanted.current;
        wanted.current = null;
        said.current.whole(next);
      })
      .catch((why) => {
        said.current.fail(refused(why));
        const next = wanted.current;
        wanted.current = null;
        if (next) opened(next);
      })
      .finally(() => {
        saving.current -= 1;
      });
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
    const at = page && placeOf(page, nth);
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

  const written = (place: Place, typed: string) => {
    const page = said.current.page();
    if (!page) return;
    canonical(page, place, typed)
      .then((said) => {
        const at = placeOf(page, place.nth);
        if (at) edited(at, [{ kind: "cell", row: place.row, column: place.column, said }]);
      })
      .catch((why) => said.current.fail(refused(why)));
  };

  const finished = (place: Place, was: string, now: string, how: Leaving) => {
    const page = said.current.page();
    const at = page && placeOf(page, place.nth);
    if (how === "cancel" || !page || !at || !fresh()) return;
    const { row, column } = place;
    const changed = now.trim() !== was.trim();
    if (row < 0) {
      if (changed) edited(at, [{ kind: "rename", column, said: now }]);
      return;
    }
    if (!changed) {
      onward(place, [], how);
      return;
    }
    // The save starts once the value is in its form, and a cell clicked meanwhile waits for it.
    saving.current += 1;
    canonical(page, place, now)
      .then((said) => onward(place, [{ kind: "cell", row, column, said }], how))
      .catch((why) => said.current.fail(refused(why)))
      .finally(() => {
        saving.current -= 1;
      });
  };

  const onward = (place: Place, cell: TableEdit[], how: Leaving) => {
    const page = said.current.page();
    const at = page && placeOf(page, place.nth);
    if (!page || !at) return;
    const { nth, row, column } = place;
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
    if (cell.length) edited(at, cell, then);
    else if (then) opened(then);
  };

  const opened = (place: Place) => {
    const page = said.current.page();
    const at = page && placeOf(page, place.nth);
    const holder = page && cellOf(page, place.nth, place.row, place.column);
    if (!page || !at || !holder || !fresh()) return;
    const type = cellOf(page, place.nth, -1, place.column)?.dataset.type ?? "";
    tableWritten(said.current.latest.current, at)
      .then((written) => {
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
            place,
          });
          return;
        }
        const name = headsNamed(page, place.nth)[place.column] ?? "";
        const label = place.row < 0 ? t("tableHead") : fill("tableCell", name);
        typing(holder, was ?? "", label, (now, how) => finished(place, was ?? "", now, how));
      })
      .catch((why) => said.current.fail(refused(why)));
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
    const next = pending.current;
    pending.current = null;
    if (next) opened(next);
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
    const at = { x: (box?.left ?? 0) + asked.x, y: (box?.top ?? 0) + asked.y };
    const name = headsNamed(page as Document, nth)[asked.column] ?? "";
    if (asked.act === "cell") {
      const place = { nth, row: asked.row, column: asked.column };
      if (saving.current > 0) wanted.current = place;
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
      setMenu({ ...at, label: t("tableRow"), choices: rowChoices(asked, nth) });
    if (asked.act === "column")
      setMenu({ ...at, label: fill("tableColumn", name), choices: columnMenu(asked, nth) });
  };

  return {
    onTable,
    menu: (
      <>
        {menu && (
          <Menu
            at={{ x: menu.x, y: menu.y }}
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
            onWrite={(typed) => {
              setPicking(null);
              written(picking.place, typed);
            }}
            onClose={() => setPicking(null)}
          />
        )}
      </>
    ),
  };
};
