import { useId, useRef } from "react";
import { t } from "../locales";
import Glyph from "./Glyph";
import { useHeldFocus } from "./heldFocus";

interface Props {
  title: string;
  wide?: boolean;
  art?: React.ReactNode;
  onClose?: () => void;
  /** An X beside the title, for a notice whose buttons all do something. */
  closable?: boolean;
  children: React.ReactNode;
}

export default function Modal({ title, wide, art, onClose, closable, children }: Props) {
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
        <div className="mb-3.5 flex items-start gap-3">
          <h2
            id={heading}
            className={`flex-1 text-[21px] font-semibold ${art ? "text-center" : ""}`}
          >
            {title}
          </h2>
          {closable && onClose && (
            <button
              type="button"
              onClick={onClose}
              aria-label={t("close")}
              title={t("close")}
              className="grid size-7 shrink-0 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
            >
              <Glyph name="close" className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
        {children}
      </div>
    </div>
  );
}
