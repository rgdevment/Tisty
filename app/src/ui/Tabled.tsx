import type { Editor as Writing } from "@tiptap/core";
import { useEffect, useState } from "react";
import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  editor: Writing;
  at: { x: number; y: number };
}

const LEANS = [
  { key: "left", icon: "alignleft" },
  { key: "center", icon: "aligncenter" },
  { key: "right", icon: "alignright" },
] as const;

export const leaning = (editor: Writing, which: string | null): boolean =>
  editor
    .chain()
    .focus()
    .command(({ tr, state }) => {
      const { $from } = state.selection;
      let cell: number | null = null;
      let column: number | null = null;
      for (let deep = $from.depth; deep > 0; deep -= 1) {
        const named = $from.node(deep).type.name;
        if (named === "tableCell" || named === "tableHeader") {
          cell = $from.before(deep);
          const index = $from.index(deep - 1);
          let grid = 0;
          $from.node(deep - 1).forEach((one, _at, spot) => {
            if (spot < index) grid += Number(one.attrs.colspan ?? 1) || 1;
          });
          column = grid;
          break;
        }
      }
      if (cell === null || column === null) return false;

      const table = state.doc.resolve(cell).node(-1) ? state.doc.resolve(cell).before(-1) : null;
      if (table === null) return false;
      const held = state.doc.nodeAt(table);
      if (!held) return false;

      let at = table + 1;
      held.forEach((row) => {
        let spot = at + 1;
        let grid = 0;
        row.forEach((one) => {
          const wide = Number(one.attrs.colspan ?? 1) || 1;
          if (grid <= column && column < grid + wide) {
            tr.setNodeAttribute(spot, "textAlign", which);
          }
          grid += wide;
          spot += one.nodeSize;
        });
        at += row.nodeSize;
      });
      return true;
    })
    .run();

export default function Tabled({ editor, at }: Props) {
  const [spot, setSpot] = useState(at);

  useEffect(() => setSpot(at), [at]);

  useEffect(() => {
    const again = () => {
      const held = editor.view.dom.querySelector<HTMLElement>("table .selectedCell, table");
      const box = held?.getBoundingClientRect();
      if (box) setSpot({ x: box.left + box.width / 2, y: box.top - 6 });
    };
    window.addEventListener("scroll", again, true);
    window.addEventListener("resize", again);
    return () => {
      window.removeEventListener("scroll", again, true);
      window.removeEventListener("resize", again);
    };
  }, [editor]);

  const leans =
    String(editor.getAttributes("tableCell").textAlign ?? "") ||
    String(editor.getAttributes("tableHeader").textAlign ?? "");

  const acts = [
    { key: "rowMore", run: () => editor.chain().focus().addRowAfter().run() },
    { key: "rowLess", run: () => editor.chain().focus().deleteRow().run() },
    { key: "colMore", run: () => editor.chain().focus().addColumnAfter().run() },
    { key: "colLess", run: () => editor.chain().focus().deleteColumn().run() },
  ] as const;

  return (
    <div
      role="toolbar"
      aria-label={t("tableIs")}
      style={{
        left: Math.max(120, Math.min(spot.x, window.innerWidth - 120)),
        top: Math.max(38, spot.y),
      }}
      className="fixed z-40 flex -translate-x-1/2 -translate-y-full items-center gap-0.5 rounded-[9px] border border-hair bg-panel px-1 py-1 shadow-lift"
    >
      {acts.map((one) => (
        <button
          key={one.key}
          type="button"
          onClick={one.run}
          className="rounded-md px-1.5 py-0.5 text-[11.5px] whitespace-nowrap text-soft hover:bg-hover hover:text-ink"
        >
          {t(one.key)}
        </button>
      ))}

      <span className="mx-0.5 h-4 w-px bg-hair" />

      {LEANS.map((one) => (
        <button
          key={one.key}
          type="button"
          title={t(`lean${one.key}` as Parameters<typeof t>[0])}
          aria-label={t(`lean${one.key}` as Parameters<typeof t>[0])}
          aria-pressed={leans === one.key}
          onClick={() => leaning(editor, leans === one.key ? null : one.key)}
          className={`grid h-6 w-6 place-items-center rounded-md hover:bg-hover hover:text-ink ${
            leans === one.key ? "bg-active text-ink" : "text-soft"
          }`}
        >
          <Glyph name={one.icon} />
        </button>
      ))}

      <span className="mx-0.5 h-4 w-px bg-hair" />

      <button
        type="button"
        title={t("tableGone")}
        aria-label={t("tableGone")}
        onClick={() => editor.chain().focus().deleteTable().run()}
        className="grid h-6 w-6 place-items-center rounded-md text-urgent hover:bg-hover"
      >
        <Glyph name="bin" />
      </button>
    </div>
  );
}
