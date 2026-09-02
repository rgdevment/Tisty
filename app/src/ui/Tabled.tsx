import type { Editor as Writing } from "@tiptap/core";
import { TableMap } from "@tiptap/pm/tables";
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
      let table: number | null = null;
      for (let deep = $from.depth; deep > 1; deep -= 1) {
        const named = $from.node(deep).type.name;
        if (named === "tableCell" || named === "tableHeader") {
          cell = $from.before(deep);
          table = $from.before(deep - 2);
          break;
        }
      }
      if (cell === null || table === null) return false;

      const held = state.doc.nodeAt(table);
      if (held?.type.name !== "table") return false;

      const start = table + 1;
      const map = TableMap.get(held);
      const index = map.map.indexOf(cell - start);
      if (index < 0) return false;
      const column = index % map.width;

      const rect = { left: column, right: column + 1, top: 0, bottom: map.height };
      for (const spot of map.cellsInRect(rect)) {
        tr.setNodeAttribute(start + spot, "textAlign", which);
      }
      return true;
    })
    .run();

export default function Tabled({ editor, at }: Props) {
  const [spot, setSpot] = useState<{ x: number; y: number } | null>(at);

  useEffect(() => setSpot(at), [at]);

  useEffect(() => {
    const again = () => {
      const held = editor.view.domAtPos(editor.state.selection.$from.pos).node;
      const at = held.nodeType === 1 ? (held as HTMLElement) : held.parentElement;
      const box = at?.closest("table")?.getBoundingClientRect();
      // Off the top of the window it would hover over somebody else's paragraph.
      setSpot(box && box.bottom > 44 ? { x: box.left + box.width / 2, y: box.top - 6 } : null);
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

  if (!spot) return null;

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
