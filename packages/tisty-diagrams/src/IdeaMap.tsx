import {
  type CSSProperties,
  type KeyboardEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import Board from "./Board";
import { useSay } from "./handing";
import {
  type Changed,
  moved,
  type Path,
  renamed,
  withChild,
  without,
  withSibling,
} from "./minding";
import { curve, HEXAGON, HUES, laidOut, type Placed } from "./mindlayout";
import type { Mind, Shape } from "./mindmap";

// A press that moves less than this is a press, not a drag.
const DRAG_FROM = 4;

const hueOf = (one: Placed) => (one.hue < 0 ? "var(--tisty-accent)" : HUES[one.hue % HUES.length]);

const outline: Record<Shape, string> = {
  plain: "rounded-md border border-transparent",
  square: "rounded-[3px] border",
  rounded: "rounded-lg border",
  circle: "rounded-full border",
  cloud: "rounded-full border border-dashed",
  bang: "rounded-sm border-2 border-dashed",
  hexagon: "rounded-md border",
};

const named = (path: Path) => path.join(".");
const pathOf = (said: string | undefined): Path | null => {
  if (said === undefined) return null;
  return said === "" ? [] : said.split(".").map(Number);
};

const lookOf = (one: Placed, chosen: boolean, over: boolean): CSSProperties => ({
  left: one.x,
  top: one.y,
  width: one.wide,
  height: one.tall,
  borderColor: one.depth === 1 || one.idea.shape !== "plain" ? hueOf(one) : undefined,
  ...(one.depth === 0 && { background: hueOf(one), color: "white" }),
  ...(one.idea.shape === "hexagon" && {
    clipPath: HEXAGON,
  }),
  boxShadow: chosen || over ? `0 0 0 2px var(--tisty-desk), 0 0 0 4px ${hueOf(one)}` : undefined,
});

/** A mind map drawn by Tisty, worked like XMind: Enter, Tab, Delete, a double press to rename, a drag to move. */
export default function IdeaMap({
  mind,
  chosen,
  fresh,
  onChoose,
  onChange,
}: Readonly<{
  mind: Mind;
  chosen: Path;
  fresh: string;
  onChoose: (path: Path) => void;
  onChange: (made: Changed) => void;
}>) {
  const t = useSay();
  const laid = useMemo(() => laidOut(mind), [mind]);
  const [renaming, setRenaming] = useState<Path | null>(null);
  const [over, setOver] = useState<Path | null>(null);
  const pressed = useRef<{ path: Path; x: number; y: number; dragging: boolean } | null>(null);
  // The browser ends a drag with a click on the idea it started from, whose place has just changed.
  const dropped = useRef(false);
  const here = useRef<HTMLDivElement>(null);

  // Keys keep working idea after idea: the chosen one takes the focus back unless the person is typing elsewhere.
  useEffect(() => {
    if (renaming) return;
    const active = document.activeElement;
    if (active && active !== document.body && !here.current?.contains(active)) return;
    here.current?.querySelector<HTMLElement>(`[data-idea="${named(chosen)}"]`)?.focus();
  }, [chosen, renaming]);

  // A new idea is named right away, as in any map editor.
  const added = (made: Changed) => {
    onChange(made);
    setRenaming(made.chosen);
  };

  const keyed = (e: KeyboardEvent) => {
    if (renaming || !(e.target as Element).closest("[data-idea]")) return;
    const acts: Record<string, () => void> = {
      Enter: () => added(withSibling(mind, chosen, fresh)),
      Tab: () => added(withChild(mind, chosen, fresh)),
      Delete: () => onChange(without(mind, chosen)),
      Backspace: () => onChange(without(mind, chosen)),
      F2: () => setRenaming(chosen),
    };
    const act = acts[e.key];
    if (!act) return;
    e.preventDefault();
    act();
  };

  const underPointer = (x: number, y: number): Path | null =>
    pathOf(document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-idea]")?.dataset.idea);

  const settle = (text: string) => {
    if (renaming) onChange(renamed(mind, renaming, text.trim() || fresh));
    setRenaming(null);
  };

  const editing = renaming ? laid.ideas.find((one) => named(one.path) === named(renaming)) : null;

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the chosen idea, whose button holds the focus.
    <div ref={here} className="relative flex min-w-0 flex-1" onKeyDown={keyed}>
      <Board
        size={laid.size}
        notice={
          <p className="absolute bottom-3 left-3 m-0 rounded-lg bg-panel px-3 py-2 text-[12px] text-faint shadow">
            {t("diagramMindKeys")}
          </p>
        }
      >
        <svg
          aria-hidden="true"
          className="absolute top-0 left-0 overflow-visible"
          width={laid.size.wide}
          height={laid.size.tall}
        >
          {laid.lines.map((line) => (
            <path
              key={`${line.to.x}-${line.to.y}`}
              d={curve(line)}
              fill="none"
              strokeWidth={2}
              strokeLinecap="round"
              stroke={HUES[line.hue % HUES.length]}
            />
          ))}
        </svg>
        {laid.ideas.map((one) => {
          const name = named(one.path);
          const isChosen = name === named(chosen);
          return (
            <button
              key={name || "root"}
              type="button"
              data-idea={name}
              aria-pressed={isChosen}
              className={`absolute box-border flex cursor-pointer items-center justify-center truncate px-2 text-[13px] whitespace-nowrap outline-none ${
                one.depth === 0 ? "rounded-full text-[15px] font-bold" : outline[one.idea.shape]
              } ${one.depth === 1 ? "bg-panel font-semibold text-ink" : ""} ${
                one.depth > 1 ? "bg-desk text-soft" : ""
              }`}
              style={lookOf(one, isChosen, over !== null && named(over) === name)}
              onClick={() => {
                if (dropped.current) dropped.current = false;
                else onChoose(one.path);
              }}
              onDoubleClick={() => setRenaming(one.path)}
              onPointerDown={(e) => {
                pressed.current = { path: one.path, x: e.clientX, y: e.clientY, dragging: false };
                dropped.current = false;
                e.currentTarget.setPointerCapture?.(e.pointerId);
              }}
              onPointerMove={(e) => {
                const held = pressed.current;
                if (!held || e.buttons === 0) return;
                const far = Math.hypot(e.clientX - held.x, e.clientY - held.y) > DRAG_FROM;
                if (!held.dragging && !far) return;
                held.dragging = true;
                setOver(underPointer(e.clientX, e.clientY));
              }}
              onPointerUp={(e) => {
                const held = pressed.current;
                pressed.current = null;
                setOver(null);
                if (!held?.dragging) return;
                dropped.current = true;
                const into = underPointer(e.clientX, e.clientY);
                const made = into ? moved(mind, held.path, into) : null;
                if (made) onChange(made);
              }}
              onPointerCancel={() => {
                pressed.current = null;
                setOver(null);
              }}
            >
              {one.idea.text}
            </button>
          );
        })}
        {editing && (
          <input
            autoFocus
            aria-label={t("diagramIdea")}
            defaultValue={editing.idea.text}
            onFocus={(e) => e.currentTarget.select()}
            className="absolute box-border rounded-md border border-accent bg-panel px-2 text-center text-[13px] text-ink outline-none"
            style={{
              left: editing.x - 20,
              top: editing.y,
              width: editing.wide + 40,
              height: editing.tall,
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") settle(e.currentTarget.value);
              if (e.key === "Escape") setRenaming(null);
              e.stopPropagation();
            }}
            onBlur={(e) => settle(e.currentTarget.value)}
          />
        )}
      </Board>
    </div>
  );
}
