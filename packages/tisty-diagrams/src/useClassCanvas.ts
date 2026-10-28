import { useMemo } from "react";
import { type ClassNode, headId } from "./ClassNodes";
import type { Classes } from "./classes";
import {
  type ClassAltered,
  type ClassPicked,
  classesPinned,
  classesPlaced,
  classRenamed,
  classReread,
  linkChanged,
} from "./classing";
import { classLanesOf, namespaceMoved, namespaceResized, reclassed } from "./classspaces";
import type { WireEdge } from "./FlowWire";
import { looksOf } from "./flowstyles";
import { around } from "./graphlayout";
import { useSay } from "./handing";
import { type FrameNode, handlesOf, type LaneNode } from "./ShapeNodes";
import { classJointsOf, classLaid, classSize } from "./scenes";
import type { Classing } from "./useClasses";
import { GROUP, useMoving } from "./useMoving";

export type ClassNaming = { kind: "class"; name: string } | { kind: "link"; index: number } | null;

/** The classes chosen, one or several. */
export const chosenClasses = (picked: ClassPicked): readonly string[] => {
  if (picked?.kind === "class") return [picked.name];
  return picked?.kind === "classes" ? picked.names : [];
};

const settled = (diagram: Classes, picked: ClassPicked): ClassAltered => ({
  body: diagram.body,
  places: diagram.places,
  picked,
});

/** What React Flow draws of a class diagram, and its classes and packages as they are dragged and sized. */
export const useClassCanvas = (
  editing: Classing,
  naming: ClassNaming,
  setNaming: (naming: ClassNaming) => void,
) => {
  const t = useSay();
  const { diagram, picked, change, pick } = editing;
  const still = useMemo(() => classLaid(diagram), [diagram]);
  const lanes = useMemo(
    () => new Map(classLanesOf(diagram).map((one) => [one.id, one])),
    [diagram],
  );
  const looks = useMemo(() => looksOf(diagram), [diagram]);
  const chosen = chosenClasses(picked);
  const chosenKey = chosen.join("\n");
  const { moving, laid, inside, moved, sizing, sized } = useMoving({
    still,
    lanes,
    groupOf: (name) => diagram.classes.find((one) => one.name === name)?.group ?? null,
    chosen,
    choose: (choice) =>
      pick(
        "several" in choice
          ? { kind: "classes", names: choice.several }
          : { kind: "class", name: choice.one },
      ),
    // Packages first, then the classes dropped in them, which may join or leave one on the way.
    settle: (spots, at) => {
      let now = classesPinned(diagram, at);
      const shifted: { name: string; x: number; y: number }[] = [];
      for (const [key, spot] of spots) {
        if (key.startsWith(GROUP))
          now = classReread(now, namespaceMoved(now, key.slice(GROUP.length), spot));
        else shifted.push({ name: key, x: spot.x, y: spot.y });
      }
      if (shifted.length) {
        now = classReread(now, classesPlaced(now, shifted, null));
        const rects = shifted.flatMap((one) => {
          const rect = at.get(one.name);
          return rect ? [[one.name, { ...rect, x: one.x, y: one.y }] as const] : [];
        });
        now = reclassed(now, new Map(rects));
      }
      const first = [...spots.keys()][0];
      if (picked?.kind === "classes") change(settled(now, picked));
      else if (first.startsWith(GROUP))
        change(settled(now, { kind: "namespace", name: first.slice(GROUP.length) }));
      else change(settled(now, { kind: "class", name: first }));
    },
    resize: (lane, box, at) => change(namespaceResized(classesPinned(diagram, at), lane, box)),
  });

  // The same objects while another class is dragged, so only the one moving is drawn again.
  // biome-ignore lint/correctness/useExhaustiveDependencies: `chosenKey` stands for `chosen`, `lanes` for `inside`
  const placedCards = useMemo(
    (): ClassNode[] =>
      diagram.classes.flatMap((box) => {
        const rect = still.get(box.name);
        if (!rect) return [];
        const size = classSize(box);
        const lane = box.group ? lanes.get(box.group) : undefined;
        return [
          {
            id: box.name,
            type: "class",
            position: inside(box.name, rect),
            parentId: lane ? `${GROUP}${lane.id}` : undefined,
            width: size.wide,
            height: size.tall,
            handles: handlesOf(size.wide, size.tall),
            selected: chosen.includes(box.name),
            data: {
              box,
              wide: size.wide,
              tall: size.tall,
              look: looks.get(box.name),
              naming: naming?.kind === "class" && naming.name === box.name,
              label: t("schemaName"),
              onPick: () => pick({ kind: "class", name: box.name }),
              onName: () => setNaming({ kind: "class", name: box.name }),
              onNamed: (text) => {
                if (text !== null && text.trim() !== box.name)
                  change(classRenamed(diagram, box.name, text));
                setNaming(null);
              },
            },
          },
        ];
      }),
    [diagram, still, lanes, looks, chosenKey, naming, t, pick, change, setNaming],
  );
  const cards = placedCards.map((one) => {
    const spot = moving.get(one.id);
    return spot ? { ...one, position: inside(one.id, spot) } : one;
  });
  const isPicked = (name: string) => picked?.kind === "namespace" && picked.name === name;
  const laneNodes = [...lanes.values()].map((lane): LaneNode => {
    const rect = { ...lane.rect, ...moving.get(`${GROUP}${lane.id}`) };
    return {
      id: `${GROUP}${lane.id}`,
      type: "lane",
      position: { x: rect.x, y: rect.y },
      width: rect.wide,
      height: rect.tall,
      zIndex: -1,
      selected: isPicked(lane.id),
      data: {
        title: lane.title,
        across: rect.wide > rect.tall,
        picked: isPicked(lane.id),
        onPick: () => pick({ kind: "namespace", name: lane.id }),
        onResizing: sizing,
        onResized: (box) => sized(lane.id, box),
      },
    };
  });
  const frames = diagram.groups.flatMap((group): FrameNode[] => {
    if (lanes.has(group.id)) return [];
    const rect = around(
      diagram.classes
        .filter((one) => one.group === group.id)
        .flatMap((one) => laid.get(one.name) ?? []),
    );
    if (!rect) return [];
    return [
      {
        id: `${GROUP}${group.id}`,
        type: "frame",
        position: { x: rect.x, y: rect.y },
        width: rect.wide,
        height: rect.tall,
        zIndex: -1,
        draggable: false,
        selectable: false,
        data: {
          title: group.id,
          picked: isPicked(group.id),
          onPick: () => pick({ kind: "namespace", name: group.id }),
        },
      },
    ];
  });
  const wires = useMemo(
    (): WireEdge[] =>
      classJointsOf(diagram).map((one, index) => ({
        id: one.key,
        source: one.from,
        target: one.to,
        type: "wire",
        selected: picked?.kind === "link" && picked.index === index,
        markerStart: one.start ? headId(one.start) : undefined,
        markerEnd: one.end ? headId(one.end) : undefined,
        data: {
          label: one.label,
          title: `${one.from} → ${one.to}`,
          dashed: one.dashed,
          thick: false,
          startLabel: one.startLabel,
          endLabel: one.endLabel,
          naming: naming?.kind === "link" && naming.index === index,
          field: t("flowLabel"),
          onPick: () => pick({ kind: "link", index }),
          onName: () => setNaming({ kind: "link", index }),
          onNamed: (label) => {
            if (label !== null && label.trim() !== one.label)
              change(linkChanged(diagram, index, { label }));
            setNaming(null);
          },
        },
      })),
    [diagram, picked, naming, t, pick, change, setNaming],
  );

  // Parents before the classes in them, as React Flow asks.
  return { nodes: [...frames, ...laneNodes, ...cards], edges: wires, laid, moved };
};
