import { t } from "../locales";

// Only a table that scrolls takes a keyboard stop, so its hidden columns are reached without a mouse.
export const sideways = (page: Document): (() => void)[] => {
  const boxes = Array.from(page.querySelectorAll<HTMLElement>(".page > .table[data-at]"));
  const marked = () => {
    for (const box of boxes) {
      if (box.scrollWidth > box.clientWidth) {
        box.tabIndex = 0;
        box.setAttribute("role", "region");
        box.setAttribute("aria-label", t("table"));
      } else {
        box.removeAttribute("tabindex");
        box.removeAttribute("role");
        box.removeAttribute("aria-label");
      }
    }
  };
  marked();
  const Seeing = page.defaultView?.ResizeObserver;
  if (!Seeing || !boxes.length) return [];
  const seeing = new Seeing(marked);
  for (const box of boxes) seeing.observe(box);
  return [() => seeing.disconnect()];
};
