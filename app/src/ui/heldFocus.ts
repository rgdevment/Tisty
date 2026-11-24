import { type RefObject, useEffect } from "react";

export const REACHABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export const tabbedInside = (box: HTMLElement, backwards: boolean): HTMLElement | null => {
  const inside = Array.from(box.querySelectorAll<HTMLElement>(REACHABLE));
  if (inside.length === 0) return null;
  const first = backwards ? inside[inside.length - 1] : inside[0];
  const edge = backwards ? inside[0] : inside[inside.length - 1];
  const at = box.ownerDocument.activeElement;
  if (at === box || !box.contains(at)) return first;
  return at === edge ? first : null;
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
    // WebKit gives a pressed button no focus, so taking it back must not roll the button from under the pointer.
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus({ preventScroll: true });
  };

  return { wrapped, held };
};
