import { type RefObject, useEffect } from "react";

export const REACHABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

// What sits over the window takes the focus in, keeps it inside and gives it back when it shuts.
export const useHeldFocus = (box: RefObject<HTMLElement | null>) => {
  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus();
    return () => came?.focus?.();
  }, [box]);

  const wrapped = (event: React.KeyboardEvent) => {
    if (event.key !== "Tab") return;
    const inside = Array.from(box.current?.querySelectorAll<HTMLElement>(REACHABLE) ?? []);
    if (inside.length === 0) return;
    const at = document.activeElement;
    const edge = event.shiftKey ? inside[0] : inside[inside.length - 1];
    if (at !== edge && at !== box.current) return;
    event.preventDefault();
    (event.shiftKey ? inside[inside.length - 1] : inside[0]).focus();
  };

  const held = (event: React.FocusEvent) => {
    if (event.currentTarget.contains(event.relatedTarget as Node | null)) return;
    if (document.activeElement !== document.body) return;
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus();
  };

  return { wrapped, held };
};
