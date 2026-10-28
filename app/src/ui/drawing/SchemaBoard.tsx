import { type KeyboardEvent, type PointerEvent, useMemo, useRef, useState } from "react";
import { t } from "../../locales";
import { button } from "../viewer/Glyphs";
import Board from "./Board";
import type { Schema } from "./schema";
import { type Box, bend, drawnOf, HEAD, ROW } from "./schemalayout";
import {
  type Altered,
  columnRemoved,
  type Picked,
  related,
  relationRemoved,
  tableAdded,
  tableMoved,
  tableRemoved,
} from "./scheming";

const END_MARKS = { one: "1", "zero-one": "0..1", "zero-many": "0..N", "one-many": "1..N" };

// A press that moves less than this is a press, not a drag.
const DRAG_FROM = 3;

type Drag =
  | {
      kind: "table";
      name: string;
      x: number;
      y: number;
      from: { x: number; y: number };
      moved: boolean;
    }
  | { kind: "link"; table: string; index: number; at: { x: number; y: number } };

// The board is scaled; a pointer's move is divided by how much, read off the drawn room itself.
const scaleOf = (inside: Element) => {
  const room = inside.closest<HTMLElement>(".diagram-drawn");
  const shown = room?.getBoundingClientRect().width ?? 0;
  return room?.offsetWidth ? shown / room.offsetWidth : 1;
};

const pointIn = (e: PointerEvent, inside: Element) => {
  const room = inside.closest<HTMLElement>(".diagram-drawn");
  const box = room?.getBoundingClientRect();
  const scale = scaleOf(inside);
  return { x: (e.clientX - (box?.left ?? 0)) / scale, y: (e.clientY - (box?.top ?? 0)) / scale };
};

const tableUnder = (x: number, y: number) =>
  document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-table]")?.dataset.table ?? null;

const same = (picked: Picked, other: NonNullable<Picked>): boolean => {
  if (picked?.kind !== other.kind) return false;
  if (other.kind === "table") return picked.kind === "table" && picked.name === other.name;
  if (other.kind === "column")
    return picked.kind === "column" && picked.table === other.table && picked.index === other.index;
  return picked.kind === "relation" && picked.index === other.index;
};

/** Tables as cards: a title to drag them by, columns to choose, and a dot to drag onto another table to relate them. */
export default function SchemaBoard({
  schema,
  picked,
  fresh,
  onPick,
  onChange,
}: Readonly<{
  schema: Schema;
  picked: Picked;
  fresh: string;
  onPick: (picked: Picked) => void;
  onChange: (made: Altered) => void;
}>) {
  const drawn = useMemo(() => drawnOf(schema), [schema]);
  const [drag, setDrag] = useState<Drag | null>(null);
  const pressed = useRef<Drag | null>(null);

  const keyed = (e: KeyboardEvent) => {
    if (e.key !== "Delete" && e.key !== "Backspace") return;
    if (!(e.target as Element).closest("[data-picks]") || !picked) return;
    e.preventDefault();
    if (picked.kind === "table") onChange(tableRemoved(schema, picked.name));
    if (picked.kind === "column") onChange(columnRemoved(schema, picked.table, picked.index));
    if (picked.kind === "relation") onChange(relationRemoved(schema, picked.index));
  };

  const shownAt = (box: Box) =>
    drag?.kind === "table" && drag.name === box.table.name ? { x: drag.x, y: drag.y } : box;

  const moving = (e: PointerEvent) => {
    const held = pressed.current;
    if (!held || e.buttons === 0) return;
    if (held.kind === "link") {
      const next = { ...held, at: pointIn(e, e.currentTarget) };
      pressed.current = next;
      setDrag(next);
      return;
    }
    const scale = scaleOf(e.currentTarget);
    const x = Math.max(0, held.from.x + (e.clientX - held.x) / scale);
    const y = Math.max(0, held.from.y + (e.clientY - held.y) / scale);
    const far = held.moved || Math.hypot(e.clientX - held.x, e.clientY - held.y) > DRAG_FROM;
    if (!far) return;
    pressed.current = { ...held, moved: true };
    setDrag({ ...held, x, y, moved: true });
  };

  const letGo = (e: PointerEvent) => {
    const held = pressed.current;
    pressed.current = null;
    setDrag(null);
    if (!held) return;
    if (held.kind === "link") {
      const into = tableUnder(e.clientX, e.clientY);
      const made = into ? related(schema, { table: held.table, index: held.index }, into) : null;
      if (made) onChange(made);
      return;
    }
    if (!held.moved) return;
    const scale = scaleOf(e.currentTarget);
    const at = {
      x: Math.max(0, held.from.x + (e.clientX - held.x) / scale),
      y: Math.max(0, held.from.y + (e.clientY - held.y) / scale),
    };
    onChange(tableMoved(schema, held.name, at));
  };

  const linking = drag?.kind === "link" ? drag : null;
  const linkFrom = linking && drawn.boxes.find((one) => one.table.name === linking.table);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: Delete belongs to what is chosen, whose button holds the focus.
    <div className="relative flex min-w-0 flex-1" onKeyDown={keyed}>
      <Board
        size={drawn.size}
        notice={
          <div className="absolute top-3 left-3 flex items-center gap-2">
            <button
              type="button"
              className={`${button} bg-panel`}
              onClick={() => onChange(tableAdded(schema, fresh))}
            >
              {t("schemaAddTable")}
            </button>
            <p className="m-0 rounded-lg bg-panel px-3 py-1.5 text-[12px] text-faint shadow">
              {t("schemaKeys")}
            </p>
          </div>
        }
      >
        <svg
          aria-hidden="true"
          className="absolute top-0 left-0 overflow-visible"
          width={drawn.size.wide}
          height={drawn.size.tall}
        >
          {drawn.links.map((one) => (
            <path
              key={one.index}
              d={bend(one)}
              fill="none"
              stroke="var(--tisty-accent)"
              strokeWidth={same(picked, { kind: "relation", index: one.index }) ? 3 : 1.6}
              strokeDasharray={one.relation.identifying ? undefined : "6 4"}
            />
          ))}
          {linking && linkFrom && (
            <path
              d={bend({
                from: {
                  x: linkFrom.x + linkFrom.wide,
                  y: linkFrom.y + HEAD + (linking.index + 0.5) * ROW,
                },
                to: linking.at,
              })}
              fill="none"
              stroke="var(--tisty-accent)"
              strokeWidth={2}
              strokeDasharray="4 4"
            />
          )}
        </svg>
        {drawn.links.map((one) => (
          <button
            key={one.index}
            type="button"
            data-picks
            aria-pressed={same(picked, { kind: "relation", index: one.index })}
            className="absolute -translate-x-1/2 -translate-y-1/2 rounded-full border border-accent bg-desk px-2 py-0.5 text-[11px] whitespace-nowrap text-accent"
            style={{ left: (one.from.x + one.to.x) / 2, top: (one.from.y + one.to.y) / 2 }}
            onClick={() => onPick({ kind: "relation", index: one.index })}
          >
            {`${END_MARKS[one.relation.fromEnd]} · ${one.relation.label || "—"} · ${END_MARKS[one.relation.toEnd]}`}
          </button>
        ))}
        {drawn.boxes.map((box) => {
          const name = box.table.name;
          const at = shownAt(box);
          const chosenTable = same(picked, { kind: "table", name });
          return (
            <div
              key={name}
              data-table={name}
              className={`absolute overflow-hidden rounded-lg border bg-panel text-[12.5px] shadow-sm ${
                chosenTable ? "border-accent" : "border-line"
              }`}
              style={{ left: at.x, top: at.y, width: box.wide }}
            >
              <button
                type="button"
                data-picks
                aria-pressed={chosenTable}
                className="flex w-full cursor-move items-center border-b border-hair bg-hover px-3 font-semibold text-ink"
                style={{ height: HEAD }}
                onClick={() => onPick({ kind: "table", name })}
                onPointerDown={(e) => {
                  pressed.current = {
                    kind: "table",
                    name,
                    x: e.clientX,
                    y: e.clientY,
                    from: at,
                    moved: false,
                  };
                  e.currentTarget.setPointerCapture?.(e.pointerId);
                }}
                onPointerMove={moving}
                onPointerUp={letGo}
                onPointerCancel={() => {
                  pressed.current = null;
                  setDrag(null);
                }}
              >
                {name}
              </button>
              {box.table.columns.map((column, index) => {
                const chosen = same(picked, { kind: "column", table: name, index });
                return (
                  <div
                    // biome-ignore lint/suspicious/noArrayIndexKey: a column is its place in the table; two may share a name while typed.
                    key={index}
                    className="relative flex items-center"
                    style={{ height: ROW }}
                  >
                    <button
                      type="button"
                      data-picks
                      aria-pressed={chosen}
                      className={`flex h-full flex-1 items-center gap-2 px-3 text-left ${chosen ? "bg-accent-soft" : ""}`}
                      onClick={() => onPick({ kind: "column", table: name, index })}
                    >
                      <span className="w-6 text-[10px] font-bold text-accent">
                        {column.keys.join(",")}
                      </span>
                      <span className="flex-1 truncate text-ink">
                        {column.name}
                        {column.required ? " *" : ""}
                      </span>
                      <span className="font-mono text-[11.5px] text-faint">{column.type}</span>
                    </button>
                    <span
                      data-grip
                      role="presentation"
                      title={t("schemaLink")}
                      className="mr-1.5 h-2.5 w-2.5 shrink-0 cursor-crosshair rounded-full border border-accent bg-desk"
                      onPointerDown={(e) => {
                        pressed.current = {
                          kind: "link",
                          table: name,
                          index,
                          at: pointIn(e, e.currentTarget),
                        };
                        e.currentTarget.setPointerCapture?.(e.pointerId);
                      }}
                      onPointerMove={moving}
                      onPointerUp={letGo}
                    />
                  </div>
                );
              })}
            </div>
          );
        })}
      </Board>
    </div>
  );
}
