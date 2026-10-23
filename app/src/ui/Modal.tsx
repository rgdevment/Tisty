import { useId, useRef } from "react";
import { useHeldFocus } from "./heldFocus";

interface Props {
  title: string;
  wide?: boolean;
  art?: React.ReactNode;
  onClose?: () => void;
  children: React.ReactNode;
}

export default function Modal({ title, wide, art, onClose, children }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const heading = useId();

  const { wrapped, held } = useHeldFocus(box);

  const kept = (event: React.KeyboardEvent) => {
    if (event.key === "Escape" && onClose) {
      event.preventDefault();
      onClose();
      return;
    }
    wrapped(event);
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
        className={`scroller max-h-full w-full ${wide ? "max-w-lg" : "max-w-md"} rounded-[10px] border border-hair bg-bg p-6 shadow-lift-tall`}
      >
        {art}
        <h2 id={heading} className={`mb-3.5 text-[21px] font-semibold ${art ? "text-center" : ""}`}>
          {title}
        </h2>
        {children}
      </div>
    </div>
  );
}
