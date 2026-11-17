import { useEffect, useLayoutEffect, useRef, useState } from "react";

export interface Pickable {
  key: string;
  label: string;
  glyph?: string;
  detail?: string;
  keys?: string;
  group?: string;
  onPick: () => void;
}

interface Keyed {
  key: string;
  preventDefault: () => void;
}

// Whoever holds the words, a field of its own or the document's caret, hands its keys here.
export function usePicker(rows: Pickable[], asked: string, onClose: () => void) {
  const [at, setAt] = useState(0);
  const on = rows.length === 0 ? -1 : Math.min(at, rows.length - 1);

  useEffect(() => setAt(0), [asked]);

  const keyed = (event: Keyed): boolean => {
    if (event.key === "Escape") onClose();
    else if (on < 0) return false;
    else if (event.key === "ArrowDown") setAt((on + 1) % rows.length);
    else if (event.key === "ArrowUp") setAt((on - 1 + rows.length) % rows.length);
    else if (event.key === "Enter") rows[on].onPick();
    else return false;
    event.preventDefault();
    return true;
  };

  return { on, setAt, keyed };
}

export const optionOf = (list: string, at: number): string | undefined =>
  at < 0 ? undefined : `${list}-${at}`;

export function PickList({
  id,
  label,
  rows,
  on,
  onHover,
}: {
  id: string;
  label: string;
  rows: Pickable[];
  on: number;
  onHover: (at: number) => void;
}) {
  const list = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const row = on < 0 ? null : list.current?.querySelector(`#${CSS.escape(`${id}-${on}`)}`);
    row?.scrollIntoView?.({ block: "nearest" });
  }, [id, on]);

  return (
    <div ref={list} id={id} role="listbox" aria-label={label} className="flex flex-col">
      {grouped(rows).map(({ group, from, all }) => (
        <div key={`${group}-${from}`} role="presentation">
          {group && (
            <p
              aria-hidden="true"
              className="px-2.5 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
            >
              {group}
            </p>
          )}
          {all.map((row, within) => {
            const at = from + within;
            return (
              <button
                type="button"
                key={row.key}
                id={optionOf(id, at)}
                role="option"
                aria-selected={at === on}
                tabIndex={-1}
                onMouseMove={() => at !== on && onHover(at)}
                // The words stay where they are typed: a click picks without taking their focus.
                onMouseDown={(event) => event.preventDefault()}
                onClick={row.onPick}
                className={`flex w-full cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[13px] ${
                  at === on ? "bg-accent-soft" : ""
                }`}
              >
                {row.glyph && (
                  <span aria-hidden="true" className="w-[15px] shrink-0 text-center text-soft">
                    {row.glyph}
                  </span>
                )}
                <span className="min-w-0 truncate text-ink">{row.label}</span>
                {row.detail && (
                  <span className="min-w-0 shrink truncate text-[11.5px] text-faint">
                    {row.detail}
                  </span>
                )}
                {row.keys && (
                  <kbd className="ml-auto shrink-0 font-sans text-[11.5px] text-faint">
                    {row.keys}
                  </kbd>
                )}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}

const grouped = (rows: Pickable[]) =>
  rows.reduce<{ group: string; from: number; all: Pickable[] }[]>((runs, row, at) => {
    const last = runs[runs.length - 1];
    const group = row.group ?? "";
    if (last && last.group === group) last.all.push(row);
    else runs.push({ group, from: at, all: [row] });
    return runs;
  }, []);
