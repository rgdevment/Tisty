import { t } from "../../locales";
import { button } from "../viewer/Glyphs";
import { classAdded, classesLinked, classRemoved, linkRemoved, memberRemoved } from "./classing";
import GraphBoard from "./GraphBoard";
import { classScene } from "./scenes";
import type { Classing } from "./useClasses";

/** A class diagram drawn by Tisty: classes as cards with their members, in layers along its relations. */
export default function ClassView({ editing }: Readonly<{ editing: Classing }>) {
  const { diagram, picked, change, pick } = editing;
  const { laid, joints } = classScene(diagram);
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
      wires={joints.map((one, index) => ({
        ...one,
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
