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
import { MindSketch } from "./MindSketch";
import {
  type Changed,
  moved,
  type Path,
  renamed,
  reshaped,
  withChild,
  without,
  withSibling,
} from "./minding";
import { curve, HEXAGON, HUES, laidOut, type Placed } from "./mindlayout";
import { type Mind, SHAPES, type Shape } from "./mindmap";
import { IDEA_LINES, SHAPE_WORDS } from "./mindshapes";
import Palette from "./Palette";
import { Gap, RemoveTools, Tool, ToolBar, UndoTools, ZoomTools } from "./tools";

// A press that moves less than this is a press, not a drag.
const DRAG_FROM = 4;

const hueOf = (one: Placed) => (one.hue < 0 ? "var(--tisty-accent)" : HUES[one.hue % HUES.length]);

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
  undo,
  redo,
  onChoose,
  onChange,
}: Readonly<{
  mind: Mind;
  chosen: Path;
  fresh: string;
  undo: (() => void) | null;
  redo: (() => void) | null;
  onChoose: (path: Path) => void;
  onChange: (made: Changed, merged?: boolean) => void;
}>) {
  const t = useSay();
  const laid = useMemo(() => laidOut(mind), [mind]);
  const [renaming, setRenaming] = useState<Path | null>(null);
  const [over, setOver] = useState<Path | null>(null);
  const pressed = useRef<{ path: Path; x: number; y: number; dragging: boolean } | null>(null);
  // The browser ends a drag with a click on the idea it started from, whose place has just changed.
  const dropped = useRef(false);
  const here = useRef<HTMLDivElement>(null);
  const room = useRef<HTMLElement>(null);
  // An idea just added is named as part of adding it, so one undo takes both away.
  const justAdded = useRef(false);

  // Keys keep working idea after idea: the chosen one takes the focus back unless the person is typing elsewhere.
  useEffect(() => {
    if (renaming) return;
    const active = document.activeElement;
    if (active && active !== document.body && !here.current?.contains(active)) return;
    here.current?.querySelector<HTMLElement>(`[data-idea="${named(chosen)}"]`)?.focus();
  }, [chosen, renaming]);

  // A new idea is named right away, as in any map editor.
  const renameOld = (path: Path | null) => {
    justAdded.current = false;
    setRenaming(path);
  };

  const added = (made: Changed) => {
    onChange(made);
    justAdded.current = true;
    setRenaming(made.chosen);
  };

  const grown = (shape: Shape, under: Path) => {
    const made = withChild(mind, under, fresh);
    added(reshaped(made.mind, made.chosen, shape));
  };

  const keyed = (e: KeyboardEvent) => {
    if (renaming) return;
    const mod = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    if (mod && (key === "z" || key === "y")) {
      e.preventDefault();
      (key === "y" || e.shiftKey ? redo : undo)?.();
      return;
    }
    if (!(e.target as Element).closest("[data-idea]")) return;
    const acts = new Map<string, () => void>([
      ["Enter", () => added(withSibling(mind, chosen, fresh))],
      ["Tab", () => added(withChild(mind, chosen, fresh))],
      ["Delete", () => onChange(without(mind, chosen))],
      ["Backspace", () => onChange(without(mind, chosen))],
      ["F2", () => renameOld(chosen)],
    ]);
    const act = acts.get(e.key);
    if (!act) return;
    e.preventDefault();
    act();
  };

  const underPointer = (x: number, y: number): Path | null =>
    pathOf(document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-idea]")?.dataset.idea);

  const settle = (text: string) => {
    if (renaming) onChange(renamed(mind, renaming, text.trim() || fresh), justAdded.current);
    justAdded.current = false;
    setRenaming(null);
  };

  const editing = renaming ? laid.ideas.find((one) => named(one.path) === named(renaming)) : null;

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the chosen idea, whose button holds the focus.
    <div ref={here} className="relative flex min-h-0 min-w-0 flex-1" onKeyDown={keyed}>
      <Palette<Shape>
        groups={[
          {
            title: t("flowPalette"),
            items: SHAPES.map((shape) => ({ key: shape, label: t(SHAPE_WORDS[shape]) })),
          },
        ]}
        sketch={(shape) => <MindSketch shape={shape} />}
        onAdd={(shape) => grown(shape, chosen)}
        onDrop={(shape, at) => {
          if (!room.current?.contains(document.elementFromPoint(at.x, at.y))) return;
          grown(shape, underPointer(at.x, at.y) ?? chosen);
        }}
      />
      <Board
        size={laid.size}
        room={room}
        bar={(zooming) => (
          <ToolBar>
            <UndoTools undo={undo} redo={redo} />
            <Gap />
            <Tool
              label={t("diagramAddChild")}
              hint={`${t("diagramAddChild")} (Tab)`}
              icon="child"
              onClick={() => added(withChild(mind, chosen, fresh))}
            />
            <Tool
              label={t("diagramAddSibling")}
              hint={`${t("diagramAddSibling")} (Enter ↵)`}
              icon="sibling"
              onClick={() => added(withSibling(mind, chosen, fresh))}
            />
            <Gap />
            <RemoveTools
              removable={chosen.length > 0}
              refused={undefined}
              onRemove={() => onChange(without(mind, chosen))}
            />
            <ZoomTools {...zooming} />
          </ToolBar>
        )}
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
              title={t("diagramIdeaKeys")}
              aria-pressed={isChosen}
              className={`absolute box-border flex cursor-pointer items-center justify-center truncate px-2 text-[13px] whitespace-nowrap outline-none ${
                one.depth === 0 ? "rounded-full text-[15px] font-bold" : IDEA_LINES[one.idea.shape]
              } ${one.depth === 1 ? "bg-panel font-semibold text-ink" : ""} ${
                one.depth > 1 ? "bg-desk text-soft" : ""
              }`}
              style={lookOf(one, isChosen, over !== null && named(over) === name)}
              onClick={() => {
                if (dropped.current) dropped.current = false;
                else onChoose(one.path);
              }}
              onDoubleClick={() => renameOld(one.path)}
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
              if (e.key === "Escape") renameOld(null);
              e.stopPropagation();
            }}
            onBlur={(e) => settle(e.currentTarget.value)}
          />
        )}
      </Board>
    </div>
  );
}
