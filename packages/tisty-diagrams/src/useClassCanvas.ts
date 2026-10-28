import type { NodeChange } from "@xyflow/react";
import { useMemo, useState } from "react";
import { type ClassNode, headId } from "./ClassNodes";
import { choiceAfter } from "./choosing";
import {
  type ClassPicked,
  classesPinned,
  classesPlaced,
  classRenamed,
  linkChanged,
} from "./classing";
import type { WireEdge } from "./FlowWire";
import { looksOf } from "./flowstyles";
import type { Rect } from "./graphlayout";
import { useSay } from "./handing";
import { handlesOf } from "./ShapeNodes";
import { classJointsOf, classLaid, classSize } from "./scenes";
import type { Classing } from "./useClasses";

export type ClassNaming = { kind: "class"; name: string } | { kind: "link"; index: number } | null;

type Point = { readonly x: number; readonly y: number };

/** The classes chosen, one or several. */
export const chosenClasses = (picked: ClassPicked): readonly string[] => {
  if (picked?.kind === "class") return [picked.name];
  return picked?.kind === "classes" ? picked.names : [];
};

/** What React Flow draws of a class diagram, and the classes as they are dragged, written once let go. */
export const useClassCanvas = (
  editing: Classing,
  naming: ClassNaming,
  setNaming: (naming: ClassNaming) => void,
) => {
  const t = useSay();
  const { diagram, picked, change, pick } = editing;
  const [moving, setMoving] = useState<ReadonlyMap<string, Point>>(new Map());
  const still = useMemo(() => classLaid(diagram), [diagram]);
  const looks = useMemo(() => looksOf(diagram), [diagram]);
  const laid = useMemo(() => {
    if (!moving.size) return still;
    const at = new Map<string, Rect>(still);
    for (const [name, point] of moving) {
      const rect = at.get(name);
      if (rect) at.set(name, { ...rect, ...point });
    }
    return at;
  }, [still, moving]);
  const chosen = chosenClasses(picked);
  const chosenKey = chosen.join("\n");

  // The same objects while another class is dragged, so only the one moving is drawn again.
  // biome-ignore lint/correctness/useExhaustiveDependencies: `chosenKey` stands for `chosen`
  const placedCards = useMemo(
    (): ClassNode[] =>
      diagram.classes.flatMap((box) => {
        const rect = still.get(box.name);
        if (!rect) return [];
        const size = classSize(box);
        return [
          {
            id: box.name,
            type: "class",
            position: { x: rect.x, y: rect.y },
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
    [diagram, still, looks, chosenKey, naming, t, pick, change, setNaming],
  );
  const cards = moving.size
    ? placedCards.map((one) => {
        const point = moving.get(one.id);
        return point ? { ...one, position: point } : one;
      })
    : placedCards;
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

  // A drag or a nudge with the arrow keys is written once it ends, at the point it ended.
  const moved = (changes: NodeChange[]) => {
    const choice = choiceAfter(chosen, changes, () => false);
    if (choice && "several" in choice) pick({ kind: "classes", names: choice.several });
    else if (choice) pick({ kind: "class", name: choice.one });
    const spots = new Map(moving);
    let touched = false;
    let ended = false;
    for (const one of changes)
      if (one.type === "position" && one.position) {
        spots.set(one.id, one.position);
        touched = true;
        ended ||= !one.dragging;
      }
    if (!touched) return;
    if (!ended) {
      setMoving(spots);
      return;
    }
    setMoving(new Map());
    const moves = [...spots].map(([name, at]) => ({ name, x: at.x, y: at.y }));
    const kept: ClassPicked =
      picked?.kind === "classes" ? picked : { kind: "class", name: moves[0].name };
    change(classesPlaced(classesPinned(diagram, laid), moves, kept));
  };

  return { nodes: cards, edges: wires, laid, moved };
};
