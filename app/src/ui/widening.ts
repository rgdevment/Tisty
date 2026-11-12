import type { Widths } from "tisty-editor/edit";
import { spanned } from "tisty-editor/read";

const STEP = 10;
const FEWEST = 4;
const LEAST = FEWEST * STEP;

export const gripped = (page: Document) => {
  for (const head of page.querySelectorAll<HTMLElement>(".page > .table[data-at] thead th")) {
    const grip = page.createElement("span");
    grip.className = "col-grip";
    grip.setAttribute("aria-hidden", "true");
    head.append(grip);
  }
};

// Dragging a column's edge is said back as dashes in its rule, ten pixels each, as the core reads them.
export const widening = (
  page: Document,
  widen: () => ((at: [number, number], widths: Widths) => void) | undefined,
) => {
  page.addEventListener("pointerdown", (event) => {
    const grip = (event.target as Element | null)?.closest?.(".col-grip");
    const head = grip?.closest("th");
    const table = head?.closest<HTMLElement>(".page > .table[data-at]");
    const at = spanned(table?.dataset.at ?? null);
    if (!grip || !head || !table || !at || event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const column = Array.from(head.parentElement?.children ?? []).indexOf(head);
    const cols = table.querySelectorAll<HTMLElement>("colgroup > col");
    const col = cols[column];
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const fit = Number(table.dataset.fit) || 1;
    const start = event.clientX;
    const was = head.offsetWidth;
    const next = fit < 1 ? cols[column + 1] : undefined;
    const beside = next ? (head.nextElementSibling as HTMLElement | null) : null;
    // A table pressed to its sheet has no width to spare, so its edge takes from the column beside.
    const trading = next && beside ? { next, both: was + beside.offsetWidth } : null;
    let wide = was;
    (grip as HTMLElement).setPointerCapture?.(event.pointerId);
    page.documentElement.classList.add("widening");
    const moved = (now: PointerEvent) => {
      const most = trading ? trading.both - LEAST : Number.POSITIVE_INFINITY;
      wide = Math.min(most, Math.max(LEAST, Math.round(was + (now.clientX - start) / zoom)));
      if (col) col.style.width = `${wide}px`;
      if (trading) trading.next.style.width = `${trading.both - wide}px`;
    };
    const ended = () => {
      page.removeEventListener("pointermove", moved);
      page.removeEventListener("pointerup", ended);
      page.removeEventListener("pointercancel", ended);
      page.documentElement.classList.remove("widening");
      if (wide === was) return;
      const dashes = (seen: number) => Math.max(FEWEST, Math.round(seen / fit / STEP));
      const widths: Widths = trading
        ? [
            [column, dashes(wide)],
            [column + 1, dashes(trading.both - wide)],
          ]
        : [[column, dashes(wide)]];
      widen()?.(at, widths);
    };
    page.addEventListener("pointermove", moved);
    page.addEventListener("pointerup", ended);
    page.addEventListener("pointercancel", ended);
  });
};
