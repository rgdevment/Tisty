import { t } from "../../locales";
import { button } from "../viewer/Glyphs";
import type { ClassBox } from "./classes";
import { classAdded, classesLinked, classRemoved, linkRemoved, memberRemoved } from "./classing";
import GraphBoard from "./GraphBoard";
import { layered } from "./graphlayout";
import type { Classing } from "./useClasses";

const HEAD = 32;
const LINE = 20;

const sizeOf = (one: ClassBox) => ({
  wide: Math.max(
    150,
    ...[one.name, ...one.members.map((m) => m.text)].map((said) => said.length * 7.2 + 36),
  ),
  tall: HEAD + Math.max(one.members.length, 1) * LINE + 10 + (one.note ? 14 : 0),
});

// The head is drawn at the end Mermaid puts it: `<|--` and `<--` on the left class.
const headLeft = (arrow: string) => arrow.startsWith("<");
// The parent or the whole goes above: it is on the right in `--|>`, `..|>`, `--*` and `--o`.
const wholeRight = (arrow: string) => /(\|>|\*|o)$/.test(arrow);

/** A class diagram drawn by Tisty: classes as cards with their members, in layers along its relations. */
export default function ClassView({ editing }: Readonly<{ editing: Classing }>) {
  const { diagram, picked, change, pick } = editing;
  const sizes = new Map(diagram.classes.map((one) => [one.name, sizeOf(one)]));
  const laid = layered(
    diagram.classes.map((one) => one.name),
    diagram.links.map((one): readonly [string, string] =>
      wholeRight(one.arrow) ? [one.right, one.left] : [one.left, one.right],
    ),
    sizes,
    diagram.direction,
  );
  const remove = () => {
    if (picked?.kind === "class") change(classRemoved(diagram, picked.name));
    if (picked?.kind === "member") change(memberRemoved(diagram, picked.name, picked.index));
    if (picked?.kind === "link") change(linkRemoved(diagram, picked.index));
  };

  return (
    <GraphBoard
      size={laid.size}
      nodes={diagram.classes.flatMap((one) => {
        const rect = laid.at.get(one.name);
        if (!rect) return [];
        return [
          {
            id: one.name,
            rect,
            picked: picked?.kind === "class" && picked.name === one.name,
            look: "flex flex-col items-stretch overflow-hidden rounded-md border border-line bg-panel text-left text-ink",
            children: (
              <>
                <span className="border-b border-hair bg-hover px-3 py-1.5 text-center text-[13px] font-semibold">
                  {one.note && (
                    <small className="block text-[10.5px] font-normal text-faint">
                      «{one.note}»
                    </small>
                  )}
                  {one.name}
                </span>
                {one.members.map((member) => (
                  <span
                    key={member.line}
                    className="truncate px-3 font-mono text-[11.5px] leading-5 text-soft"
                  >
                    {member.text}
                  </span>
                ))}
              </>
            ),
          },
        ];
      })}
      wires={diagram.links.map((one, index) => ({
        key: String(index),
        from: headLeft(one.arrow) ? one.right : one.left,
        to: headLeft(one.arrow) ? one.left : one.right,
        label: [one.label, one.arrow].filter(Boolean).join(" "),
        dashed: one.arrow.includes("."),
        thick: false,
        arrow: /[<>]/.test(one.arrow),
        picked: picked?.kind === "link" && picked.index === index,
      }))}
      frames={[]}
      linkHint={t("classLink")}
      notice={
        <div className="absolute top-3 left-3 flex items-center gap-2">
          <button
            type="button"
            className={`${button} bg-panel`}
            onClick={() => change(classAdded(diagram, t("classNew")))}
          >
            {t("classAdd")}
          </button>
          <p className="m-0 rounded-lg bg-panel px-3 py-1.5 text-[12px] text-faint shadow">
            {t("classKeys")}
          </p>
        </div>
      }
      onPickNode={(name) => pick({ kind: "class", name })}
      onPickWire={(key) => pick({ kind: "link", index: Number(key) })}
      onPickFrame={() => {}}
      onLink={(from, to) => change(classesLinked(diagram, from, to))}
      onDelete={remove}
    />
  );
}
