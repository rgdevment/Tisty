import type { CSSProperties } from "react";
import { t } from "../../locales";
import { button } from "../viewer/Glyphs";
import type { Shape } from "./flow";
import { edgeRemoved, groupAdded, groupRemoved, linked, nodeAdded, nodeRemoved } from "./flowing";
import GraphBoard from "./GraphBoard";
import { around, layered } from "./graphlayout";
import type { Flowing } from "./useFlow";

const LOOKS: Record<Shape, string> = {
  process: "rounded-sm border",
  rounded: "rounded-xl border",
  stadium: "rounded-full border",
  subroutine: "rounded-sm border-x-4 border-y border-double",
  database: "rounded-[50%/18%] border",
  circle: "rounded-full border",
  double: "rounded-full border-4 border-double",
  decision: "",
  hexagon: "",
  lean: "",
  leanLeft: "",
  trapezoid: "",
  trapezoidDown: "",
  flag: "",
};

// Shapes CSS has no border for are cut out of a filled box instead.
const CUTS: Partial<Record<Shape, string>> = {
  decision: "polygon(50% 0, 100% 50%, 50% 100%, 0 50%)",
  hexagon: "polygon(12% 0, 88% 0, 100% 50%, 88% 100%, 12% 100%, 0 50%)",
  lean: "polygon(12% 0, 100% 0, 88% 100%, 0 100%)",
  leanLeft: "polygon(0 0, 88% 0, 100% 100%, 12% 100%)",
  trapezoid: "polygon(12% 0, 88% 0, 100% 100%, 0 100%)",
  trapezoidDown: "polygon(0 0, 100% 0, 88% 100%, 12% 100%)",
  flag: "polygon(0 0, 100% 0, 100% 100%, 0 100%, 12% 50%)",
};

const sizeOf = (text: string, shape: Shape) => {
  const wide = Math.max(90, Math.round(text.length * 7.5 + 36));
  if (shape === "circle" || shape === "double")
    return { wide: Math.max(wide, 72), tall: Math.max(wide, 72) };
  if (shape === "decision") return { wide: wide + 40, tall: 64 };
  return { wide, tall: 42 };
};

/** A flowchart drawn by Tisty: nodes in layers along its direction, groups framed, joined by dragging a dot. */
export default function FlowView({ editing }: Readonly<{ editing: Flowing }>) {
  const { flow, picked, change, pick } = editing;
  const sizes = new Map(
    flow.nodes.map((one) => [one.id, sizeOf(one.text, one.shape ?? "process")]),
  );
  const laid = layered(
    flow.nodes.map((one) => one.id),
    flow.edges.map((one) => [one.from, one.to] as const),
    sizes,
    flow.direction,
  );
  const frames = flow.groups.flatMap((group) => {
    const rects = flow.nodes
      .filter((one) => one.group === group.id)
      .flatMap((one) => laid.at.get(one.id) ?? []);
    const rect = around(rects);
    return rect
      ? [
          {
            key: group.id,
            rect,
            title: group.title,
            picked: picked?.kind === "group" && picked.id === group.id,
          },
        ]
      : [];
  });
  const remove = () => {
    if (picked?.kind === "node") change(nodeRemoved(flow, picked.id));
    if (picked?.kind === "edge") change(edgeRemoved(flow, picked.index));
    if (picked?.kind === "group") change(groupRemoved(flow, picked.id));
  };
  const look = (shape: Shape | null): { look: string; style?: CSSProperties } => {
    const cut = CUTS[shape ?? "process"];
    if (cut) return { look: "bg-accent-soft text-ink", style: { clipPath: cut } };
    return { look: `${LOOKS[shape ?? "process"]} border-line bg-panel text-ink` };
  };

  return (
    <GraphBoard
      size={laid.size}
      nodes={flow.nodes.flatMap((one) => {
        const rect = laid.at.get(one.id);
        if (!rect) return [];
        return [
          {
            id: one.id,
            rect,
            picked: picked?.kind === "node" && picked.id === one.id,
            ...look(one.shape),
            children: <span className="block truncate px-3 text-[13px]">{one.text}</span>,
          },
        ];
      })}
      wires={flow.edges.map((one, index) => ({
        key: String(index),
        from: one.from,
        to: one.to,
        label: one.label,
        dashed: one.arrow.includes("."),
        thick: one.arrow.includes("="),
        arrow: /[>ox]$/.test(one.arrow),
        picked: picked?.kind === "edge" && picked.index === index,
      }))}
      frames={frames}
      linkHint={t("flowLink")}
      notice={
        <div className="absolute top-3 left-3 flex items-center gap-2">
          <button
            type="button"
            className={`${button} bg-panel`}
            onClick={() => change(nodeAdded(flow, t("flowNewNode"), null))}
          >
            {t("flowAddNode")}
          </button>
          <button
            type="button"
            className={`${button} bg-panel`}
            onClick={() => change(groupAdded(flow, t("flowNewGroup")))}
          >
            {t("flowAddGroup")}
          </button>
          <p className="m-0 rounded-lg bg-panel px-3 py-1.5 text-[12px] text-faint shadow">
            {t("flowKeys")}
          </p>
        </div>
      }
      onPickNode={(id) => pick({ kind: "node", id })}
      onPickWire={(key) => pick({ kind: "edge", index: Number(key) })}
      onPickFrame={(id) => pick({ kind: "group", id })}
      onLink={(from, to) => change(linked(flow, from, to))}
      onDelete={remove}
    />
  );
}
