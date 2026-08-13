import { useEffect, useId, useRef } from "react";

interface Props {
  title: string;
  onClose?: () => void;
  children: React.ReactNode;
}

const REACHABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export default function Modal({ title, onClose, children }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const heading = useId();

  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    const first = box.current?.querySelector<HTMLElement>(REACHABLE);
    first?.focus();

    return () => came?.focus?.();
  }, []);

  const kept = (event: React.KeyboardEvent) => {
    if (event.key === "Escape" && onClose) {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== "Tab") return;

    const inside = Array.from(box.current?.querySelectorAll<HTMLElement>(REACHABLE) ?? []);
    if (inside.length === 0) return;
    const edge = event.shiftKey ? inside[0] : inside[inside.length - 1];
    if (document.activeElement !== edge) return;

    event.preventDefault();
    (event.shiftKey ? inside[inside.length - 1] : inside[0]).focus();
  };

  const held = (event: React.FocusEvent) => {
    if (event.currentTarget.contains(event.relatedTarget as Node | null)) return;
    if (document.activeElement !== document.body) return;
    box.current?.querySelector<HTMLElement>(REACHABLE)?.focus();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={heading}
      className="fixed inset-0 z-50 flex items-center justify-center bg-veil p-6"
      onKeyDown={kept}
      onBlur={held}
    >
      <div
        ref={box}
        className="w-full max-w-md rounded-xl border border-hair bg-bg p-6 shadow-lift-tall"
      >
        <h2 id={heading} className="text-lg font-semibold">
          {title}
        </h2>
        {children}
      </div>
    </div>
  );
}
