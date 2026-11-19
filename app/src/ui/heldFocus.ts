import { type RefObject, useEffect } from "react";

export const REACHABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export const tabbedInside = (
  box: HTMLElement,
  backwards: boolean,
  stop: HTMLElement = box,
): HTMLElement | null => {
  const inside = Array.from(box.querySelectorAll<HTMLElement>(REACHABLE));
  if (inside.length === 0) return null;
  const first = backwards ? inside[inside.length - 1] : inside[0];
  const edge = backwards ? inside[0] : inside[inside.length - 1];
  const at = box.ownerDocument.activeElement;
  if (at === stop || at === box || !box.contains(at)) return first;
  if (at !== edge) return null;
  return stop === box ? first : stop;
};

// What sits over the window takes the focus in, keeps it inside and gives it back when it shuts.
export const useHeldFocus = (box: RefObject<HTMLElement | null>) => {
  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus();
    return () => came?.focus?.();
  }, [box]);

  const wrapped = (event: React.KeyboardEvent) => {
    if (event.key !== "Tab" || !box.current) return;
    const next = tabbedInside(box.current, event.shiftKey);
    if (!next) return;
    event.preventDefault();
    next.focus();
  };

  const held = (event: React.FocusEvent) => {
    if (event.currentTarget.contains(event.relatedTarget as Node | null)) return;
    if (document.activeElement !== document.body) return;
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus();
  };

  return { wrapped, held };
};
