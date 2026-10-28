import type { CSSProperties } from "react";
import { edgeRemoved, groupAdded, groupRemoved, linked, nodeAdded, nodeRemoved } from "./flowing";
import type { Shape } from "./flowshapes";
import GraphBoard from "./GraphBoard";
import { useHands, useSay } from "./handing";
import { CUTS, flowScene } from "./scenes";
import type { Flowing } from "./useFlow";

const LOOKS: Partial<Record<Shape, string>> = {
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

/** A flowchart drawn by Tisty: nodes in layers along its direction, groups framed, joined by dragging a dot. */
export default function FlowView({ editing }: Readonly<{ editing: Flowing }>) {
  const t = useSay();
  const { button } = useHands().looks;
  const { flow, picked, change, pick } = editing;
  const { laid, joints, groups } = flowScene(flow);
  const frames = groups.map((one) => ({
    ...one,
    picked: picked?.kind === "group" && picked.id === one.key,
  }));
  const remove = () => {
    if (picked?.kind === "node") change(nodeRemoved(flow, picked.id));
    if (picked?.kind === "edge") change(edgeRemoved(flow, picked.index));
    if (picked?.kind === "group") change(groupRemoved(flow, picked.id));
  };
  const look = (shape: Shape | null): { look: string; style?: CSSProperties } => {
    const cut = CUTS[shape ?? "process"];
    if (cut) return { look: "bg-accent-soft text-ink", style: { clipPath: cut } };
    // Shapes this board cannot draw yet are shown as a process; their text keeps what they are.
    return { look: `${LOOKS[shape ?? "process"] ?? LOOKS.process} border-line bg-panel text-ink` };
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
      wires={joints.map((one, index) => ({
        ...one,
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
