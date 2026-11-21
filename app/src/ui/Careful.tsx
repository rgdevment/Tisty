import { type ReactNode, useCallback, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { t } from "../locales";
import type { Danger } from "./dangers";
import Modal from "./Modal";
import { mild } from "./Rows";

const going =
  "rounded-[10px] bg-urgent px-3 py-1 text-[12.5px] text-bg disabled:border-hair disabled:bg-hair disabled:text-soft";

export function Careful({
  danger,
  onAnswer,
}: {
  danger: Danger;
  onAnswer: (yes: boolean) => void;
}) {
  const [understood, setUnderstood] = useState(false);
  const box = useId();
  return createPortal(
    <Modal title={danger.title} onClose={() => onAnswer(false)}>
      <p className="text-[13px] leading-relaxed text-soft">{danger.lead}</p>
      {danger.points.length > 0 && (
        <ul className="mt-2.5 list-disc pl-5 text-[12.5px] leading-relaxed text-soft">
          {danger.points.map((one) => (
            <li key={one}>{one}</li>
          ))}
        </ul>
      )}
      {danger.undone && (
        <label htmlFor={box} className="mt-3.5 flex items-center gap-2 text-[12.5px] text-ink">
          <input
            id={box}
            type="checkbox"
            checked={understood}
            onChange={(e) => setUnderstood(e.target.checked)}
            className="size-[13px] accent-urgent"
          />
          {t("carefulUndone")}
        </label>
      )}
      <div className="mt-4 flex justify-end gap-2">
        <button type="button" onClick={() => onAnswer(false)} className={mild}>
          {t("cancel")}
        </button>
        <button
          type="button"
          disabled={danger.undone && !understood}
          onClick={() => onAnswer(true)}
          className={going}
        >
          {danger.go}
        </button>
      </div>
    </Modal>,
    document.body,
  );
}

export function useCareful(): { ask: (danger: Danger) => Promise<boolean>; shown: ReactNode } {
  const [asked, setAsked] = useState<Danger | null>(null);
  const settle = useRef<(yes: boolean) => void>(undefined);
  const ask = useCallback(
    (danger: Danger) =>
      new Promise<boolean>((resolve) => {
        settle.current?.(false);
        settle.current = resolve;
        setAsked(danger);
      }),
    [],
  );
  const answer = (yes: boolean) => {
    settle.current?.(yes);
    settle.current = undefined;
    setAsked(null);
  };
  return {
    ask,
    shown: asked && <Careful key={asked.title} danger={asked} onAnswer={answer} />,
  };
}
