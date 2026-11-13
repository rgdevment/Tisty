import type { EditorView } from "@codemirror/view";
import { applied, tableEdited } from "./celling";
import { type EditWord, hosted, type TableEdit } from "./hosted";
import { TYPES } from "./tableTypes";
import { widened } from "./tabling";

type Item =
  | {
      readonly word: EditWord;
      readonly edit: TableEdit;
      readonly danger?: boolean;
      readonly on?: boolean;
    }
  | { readonly heading: EditWord }
  | "apart";

const STEP_PX = 10;
const FEWEST_DASHES = 4;

const shown = (view: EditorView, anchor: HTMLElement, items: Item[], from: number) => {
  const page = view.dom.ownerDocument;
  view.dom.querySelector(".cm-table-menu")?.remove();
  const say = view.state.facet(hosted).say;
  const menu = page.createElement("div");
  menu.className = "cm-table-menu";
  menu.setAttribute("role", "menu");
  const close = () => {
    menu.remove();
    page.removeEventListener("mousedown", away, true);
    page.removeEventListener("keydown", keyed, true);
  };
  const away = (event: Event) => {
    if (!menu.contains(event.target as Node)) close();
  };
  const keyed = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    close();
  };
  for (const one of items) {
    if (one === "apart") {
      menu.append(Object.assign(page.createElement("div"), { className: "cm-table-menu-apart" }));
      continue;
    }
    if ("heading" in one) {
      menu.append(
        Object.assign(page.createElement("div"), {
          className: "cm-table-menu-heading",
          textContent: say(one.heading),
        }),
      );
      continue;
    }
    const button = page.createElement("button");
    button.type = "button";
    button.setAttribute("role", "menuitem");
    button.textContent = `${say(one.word)}${one.on ? "  ✓" : ""}`;
    if (one.danger) button.classList.add("danger");
    button.addEventListener("mousedown", (event) => {
      event.preventDefault();
      event.stopPropagation();
      close();
      tableEdited(view, from, one.edit, () => {});
    });
    menu.append(button);
  }
  const base = view.dom.getBoundingClientRect();
  const at = anchor.getBoundingClientRect();
  menu.style.left = `${at.left - base.left}px`;
  menu.style.top = `${at.bottom - base.top + 4}px`;
  view.dom.append(menu);
  const room = page.defaultView?.innerHeight ?? 0;
  const over = menu.getBoundingClientRect().bottom - room + 8;
  if (over > 0) menu.style.top = `${Math.max(8 - base.top, at.bottom - base.top + 4 - over)}px`;
  setTimeout(() => {
    page.addEventListener("mousedown", away, true);
    page.addEventListener("keydown", keyed, true);
  });
};

const countOf = (box: Element, selector: string) => box.querySelectorAll(selector).length;

export const columnAsked = (view: EditorView, asked: HTMLElement, from: number) => {
  const column = Number(asked.dataset.column ?? 0);
  const box = asked.closest("table");
  if (!box) return;
  const many = countOf(box, "thead th");
  const type = asked.closest("th")?.dataset.type ?? "text";
  const moves: Item[] = [
    ...(column > 0
      ? [
          {
            word: "tableColumnMoveLeft" as const,
            edit: { kind: "columnMove", from: column, to: column - 1 } as const,
          },
        ]
      : []),
    ...(column < many - 1
      ? [
          {
            word: "tableColumnMoveRight" as const,
            edit: { kind: "columnMove", from: column, to: column + 1 } as const,
          },
        ]
      : []),
  ];
  shown(
    view,
    asked,
    [
      { word: "tableSortAsc", edit: { kind: "sort", column, down: false } },
      { word: "tableSortDesc", edit: { kind: "sort", column, down: true } },
      "apart",
      { word: "tableColumnLeft", edit: { kind: "columnAdd", at: column } },
      { word: "tableColumnRight", edit: { kind: "columnAdd", at: column + 1 } },
      ...moves,
      "apart",
      { heading: "tableType" },
      ...TYPES.map(([words, word]) => ({
        word,
        edit: { kind: "retype", column, words } as const,
        on: words === type,
      })),
      "apart",
      { word: "tableColumnRemove", edit: { kind: "columnRemove", column }, danger: true },
    ],
    from,
  );
};

export const rowAsked = (view: EditorView, asked: HTMLElement, from: number) => {
  const row = Number(asked.dataset.row ?? 0);
  const box = asked.closest("table");
  if (!box) return;
  const many = countOf(box, "tbody > tr");
  shown(
    view,
    asked,
    [
      { word: "tableRowAbove", edit: { kind: "rowAdd", at: row } },
      { word: "tableRowBelow", edit: { kind: "rowAdd", at: row + 1 } },
      ...(row > 0
        ? [
            {
              word: "tableRowUp" as const,
              edit: { kind: "rowMove", from: row, to: row - 1 } as const,
            },
          ]
        : []),
      ...(row < many - 1
        ? [
            {
              word: "tableRowDown" as const,
              edit: { kind: "rowMove", from: row, to: row + 1 } as const,
            },
          ]
        : []),
      "apart",
      { word: "tableRowRemove", edit: { kind: "rowRemove", row }, danger: true },
    ],
    from,
  );
};

// A column's edge is written back as dashes in its rule, ten pixels each, as the core reads them.
export const columnWidened = (
  view: EditorView,
  grip: HTMLElement,
  event: MouseEvent,
  span: { from: number; to: number },
) => {
  const head = grip.closest("th");
  const box = head?.closest("table");
  if (!head || !box) return;
  const page = view.dom.ownerDocument;
  const column = head.cellIndex;
  const col = box.querySelectorAll<HTMLElement>("colgroup > col")[column];
  const start = event.clientX;
  const was = head.offsetWidth;
  let wide = was;
  page.documentElement.classList.add("widening");
  const moved = (next: MouseEvent) => {
    wide = Math.max(FEWEST_DASHES * STEP_PX, Math.round(was + next.clientX - start));
    if (col) col.style.width = `${wide}px`;
    else head.style.width = `${wide}px`;
  };
  const ended = () => {
    page.removeEventListener("mousemove", moved);
    page.removeEventListener("mouseup", ended);
    page.documentElement.classList.remove("widening");
    if (wide === was) return;
    const text = view.state.doc.toString();
    const next = widened(
      text,
      [span.from, span.to],
      column,
      Math.max(FEWEST_DASHES, Math.round(wide / STEP_PX)),
    );
    if (next !== null) applied(view, next);
  };
  page.addEventListener("mousemove", moved);
  page.addEventListener("mouseup", ended);
};
