import { ARROWS, autoTurned, edgeChanged, isAuto, nodesPlaced, pinned, reread } from "./flowing";
import { nodeStyled } from "./flowstyles";
import { alignedMoves, relaid } from "./flowtools";
import type { Direction, Rect } from "./graphlayout";
import { useSay } from "./handing";
import { ARROW_WORDS } from "./paletteshapes";
import {
  AlignTools,
  Gap,
  PaintTools,
  RemoveTools,
  Tool,
  ToolBar,
  UndoTools,
  ViewTools,
} from "./tools";
import type { Flowing } from "./useFlow";
import type { Word } from "./words";

// The ways a flowchart can run, each a way to lay it out again.
const DIRECTIONS: readonly (readonly [Direction, Word])[] = [
  ["TB", "flowDown"],
  ["LR", "flowRight"],
  ["BT", "flowUp"],
  ["RL", "flowLeft"],
];

/** What can be done to the flowchart and to what is chosen in it, above the canvas. */
export default function FlowToolbar({
  editing,
  laid,
  onRemove,
  removable,
  onDuplicate,
}: Readonly<{
  editing: Flowing;
  laid: ReadonlyMap<string, Rect>;
  onRemove: () => void;
  removable: boolean;
  onDuplicate: (() => void) | null;
}>) {
  const t = useSay();
  const { flow, picked, change, undo, redo } = editing;
  const ids = picked?.kind === "node" ? [picked.id] : picked?.kind === "nodes" ? picked.ids : [];
  const edge = picked?.kind === "edge" ? flow.edges[picked.index] : undefined;

  return (
    <ToolBar>
      <UndoTools undo={undo} redo={redo} />
      <Gap />
      <RemoveTools
        removable={removable}
        refused={picked && !removable ? t("flowChained") : undefined}
        onRemove={onRemove}
        onDuplicate={onDuplicate}
      />
      <Gap />
      <PaintTools
        disabled={!ids.length}
        onStyle={(look) => {
          const now = ids.reduce((all, id) => reread(all, nodeStyled(all, id, look)), flow);
          change({ body: now.body, places: now.places, picked });
        }}
      />
      <Gap />
      {ARROWS.map((arrow) => (
        <Tool
          key={arrow}
          label={t(ARROW_WORDS[arrow])}
          icon={arrow}
          disabled={!edge}
          pressed={edge ? edge.arrow === arrow : undefined}
          onClick={() =>
            picked?.kind === "edge" && change(edgeChanged(flow, picked.index, { arrow }))
          }
        />
      ))}
      <Gap />
      <AlignTools
        disabled={ids.length < 2}
        onAlign={(how) =>
          change(nodesPlaced(pinned(flow, laid), alignedMoves(laid, ids, how), picked))
        }
      />
      <Gap />
      <Tool
        label={t("flowAuto")}
        icon="auto"
        pressed={isAuto(flow.body)}
        onClick={() => change(autoTurned(flow, !isAuto(flow.body), laid))}
      />
      <ViewTools
        laidOut={flow.nodes.length > 0}
        onRelay={DIRECTIONS.map(([direction, word]) => ({
          label: t(word),
          onPick: () => change(relaid(flow, direction)),
        }))}
      />
    </ToolBar>
  );
}
