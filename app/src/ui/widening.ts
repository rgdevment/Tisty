import { spanned } from "./paging";

const STEP = 10;
const FEWEST = 4;

export const gripped = (page: Document) => {
  for (const head of page.querySelectorAll<HTMLElement>(".page > table[data-at] thead th")) {
    const grip = page.createElement("span");
    grip.className = "col-grip";
    grip.setAttribute("aria-hidden", "true");
    head.append(grip);
  }
};

// Dragging a column's edge is said back as dashes in its rule, ten pixels each, as the core reads them.
export const widening = (
  page: Document,
  widen: () => ((at: [number, number], column: number, dashes: number) => void) | undefined,
) => {
  page.addEventListener("pointerdown", (event) => {
    const grip = (event.target as Element | null)?.closest?.(".col-grip");
    const head = grip?.closest("th");
    const table = head?.closest<HTMLElement>(".page > table[data-at]");
    const at = spanned(table?.dataset.at ?? null);
    if (!grip || !head || !table || !at || event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const column = Array.from(head.parentElement?.children ?? []).indexOf(head);
    const col = table.querySelectorAll<HTMLElement>("colgroup > col")[column];
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const start = event.clientX;
    const was = head.offsetWidth;
    let wide = was;
    (grip as HTMLElement).setPointerCapture?.(event.pointerId);
    page.documentElement.classList.add("widening");
    const moved = (next: PointerEvent) => {
      wide = Math.max(FEWEST * STEP, Math.round(was + (next.clientX - start) / zoom));
      if (col) col.style.width = `${wide}px`;
    };
    const ended = () => {
      page.removeEventListener("pointermove", moved);
      page.removeEventListener("pointerup", ended);
      page.removeEventListener("pointercancel", ended);
      page.documentElement.classList.remove("widening");
      if (wide !== was) widen()?.(at, column, Math.max(FEWEST, Math.round(wide / STEP)));
    };
    page.addEventListener("pointermove", moved);
    page.addEventListener("pointerup", ended);
    page.addEventListener("pointercancel", ended);
  });
};
