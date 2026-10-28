import {
  ARROWS,
  edgeChanged,
  edgeRemovable,
  edgeRemoved,
  groupRemoved,
  groupRenamed,
  nodeAdded,
  nodeChanged,
  nodeMoved,
  nodeRemoved,
} from "./flowing";
import { pictured } from "./flowshapes";
import { useHands, useSay } from "./handing";
import { ARROW_WORDS, SHAPES, shapeName } from "./paletteshapes";
import { Chosen, Said, Section } from "./panelbits";
import type { Flowing } from "./useFlow";

const NONE = "";

/** What is chosen in the flowchart, to rename and reshape, and which way the whole chart runs. */
export default function FlowPanel({ editing }: Readonly<{ editing: Flowing }>) {
  const t = useSay();
  const { button } = useHands().looks;
  const { flow, picked, change } = editing;
  const node = picked?.kind === "node" ? flow.nodes.find((one) => one.id === picked.id) : undefined;
  const edge = picked?.kind === "edge" ? flow.edges[picked.index] : undefined;
  const group =
    picked?.kind === "group" ? flow.groups.find((one) => one.id === picked.id) : undefined;
  const removing = node ? nodeRemoved(flow, node.id) : null;
  const shape = node?.shape ?? "process";
  // A shape written by hand that the list does not offer is still shown, by Mermaid's name for it.
  const shapes = (SHAPES.includes(shape) ? SHAPES : [...SHAPES, shape]).map((one) => ({
    value: one,
    label: shapeName(one, t),
  }));
  const arrows = (ARROWS as readonly string[]).includes(edge?.arrow ?? "")
    ? ARROWS.map((one) => ({ value: one as string, label: t(ARROW_WORDS[one]) }))
    : [
        ...ARROWS.map((one) => ({ value: one as string, label: t(ARROW_WORDS[one]) })),
        { value: edge?.arrow ?? "", label: edge?.arrow ?? "" },
      ];

  return (
    <div className="flex flex-col gap-3">
      {!node && !edge && !group && (
        <p className="m-0 text-[12.5px] text-faint">
          {picked?.kind === "nodes"
            ? t("flowPickedMany", String(picked.ids.length))
            : t("flowPick")}
        </p>
      )}
      {node && (
        <Section title={t("flowNode")}>
          <Said
            id="flow-text"
            label={t("flowText")}
            value={node.text}
            onSay={(text) => change(nodeChanged(flow, node.id, { text }))}
          />
          {!pictured(node.keys) && (
            <Chosen
              id="flow-shape"
              label={t("flowShape")}
              value={shape}
              options={shapes}
              onChoose={(shape) => change(nodeChanged(flow, node.id, { shape }))}
            />
          )}
          <Chosen
            id="flow-group"
            label={t("flowGroup")}
            value={node.group ?? NONE}
            options={[
              { value: NONE, label: t("flowNoGroup") },
              ...flow.groups.map((one) => ({ value: one.id, label: one.title })),
            ]}
            onChoose={(into) => change(nodeMoved(flow, node.id, into || null))}
          />
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            disabled={!removing}
            title={removing ? undefined : t("flowChained")}
            onClick={() => change(removing)}
          >
            {t("flowRemoveNode")}
          </button>
        </Section>
      )}
      {edge && picked?.kind === "edge" && (
        <Section title={`${t("flowEdge")} · ${edge.from} → ${edge.to}`}>
          <Said
            id="flow-label"
            label={t("flowLabel")}
            value={edge.label}
            onSay={(label) => change(edgeChanged(flow, picked.index, { label }))}
          />
          <Chosen
            id="flow-arrow"
            label={t("flowArrow")}
            value={edge.arrow}
            options={arrows}
            onChoose={(arrow) => change(edgeChanged(flow, picked.index, { arrow }))}
          />
          <button
            type="button"
            className={`${button} self-start text-urgent`}
            disabled={!edgeRemovable(edge)}
            title={edgeRemovable(edge) ? undefined : t("flowChained")}
            onClick={() => change(edgeRemoved(flow, picked.index))}
          >
            {t("flowRemoveEdge")}
          </button>
        </Section>
      )}
      {group && (
        <Section title={t("flowGroupTitle")}>
          <Said
            id="flow-group-title"
            label={t("flowText")}
            value={group.title}
            onSay={(title) => change(groupRenamed(flow, group.id, title))}
          />
          <div className="flex gap-1.5">
            <button
              type="button"
              className={`${button} flex-1`}
              onClick={() => change(nodeAdded(flow, t("flowNewNode"), group.id))}
            >
              {t("flowAddNodeHere")}
            </button>
            <button
              type="button"
              className={`${button} text-urgent`}
              onClick={() => change(groupRemoved(flow, group.id))}
            >
              {t("flowUngroup")}
            </button>
          </div>
        </Section>
      )}
    </div>
  );
}
