import { useMemo, useState } from "react";
import { type Flow, readFlow, writeFlow } from "./flow";
import type { FlowAltered, FlowPicked } from "./flowing";
import { kindOf } from "./framing";

export interface Flowing {
  readonly flow: Flow;
  readonly picked: FlowPicked;
  readonly change: (made: FlowAltered | null) => void;
  readonly pick: (picked: FlowPicked) => void;
  readonly turn: (head: readonly string[]) => void;
  readonly undo: (() => void) | null;
  readonly redo: (() => void) | null;
}

// What the window changed, kept to be undone; text typed by hand in between ends the trail.
interface Step {
  readonly before: string;
  readonly after: string;
}

const KEPT_STEPS = 100;

// An edge is what joins two nodes, told apart from a twin by which of them it is.
const edgeSigns = (flow: Flow) => {
  const seen = new Map<string, number>();
  return flow.edges.map((one) => {
    const said = `${one.from}\n${one.to}`;
    const nth = seen.get(said) ?? 0;
    seen.set(said, nth + 1);
    return `${said}\n${nth}`;
  });
};

const signOf = (flow: Flow, picked: FlowPicked): string | null => {
  if (!picked) return null;
  if (picked.kind === "edge") return edgeSigns(flow)[picked.index] ?? null;
  if (picked.kind === "nodes")
    return picked.ids.every((id) => flow.nodes.some((one) => one.id === id))
      ? `nodes\n${picked.ids.join("\n")}`
      : null;
  const known =
    picked.kind === "node"
      ? flow.nodes.some((one) => one.id === picked.id)
      : flow.groups.some((one) => one.id === picked.id);
  return known ? `${picked.kind}\n${picked.id}` : null;
};

// Text typed by hand can move what was chosen; it is found again by what it is, or nothing is chosen.
const foundAgain = (flow: Flow, picked: FlowPicked, sign: string | null): FlowPicked => {
  if (!picked || sign === null) return null;
  if (signOf(flow, picked) === sign) return picked;
  if (picked.kind !== "edge") return null;
  const index = edgeSigns(flow).indexOf(sign);
  return index >= 0 ? { kind: "edge", index } : null;
};

/** The text read as a flowchart, while it is one the window can keep; every change is written back into the text. */
export const useFlow = (text: string, setText: (text: string) => void): Flowing | null => {
  const flow = useMemo(() => (kindOf(text) === "flowchart" ? readFlow(text) : null), [text]);
  const [chosen, setChosen] = useState<{ picked: FlowPicked; sign: string | null }>({
    picked: null,
    sign: null,
  });
  const [steps, setSteps] = useState<{ done: readonly Step[]; undone: readonly Step[] }>({
    done: [],
    undone: [],
  });
  if (!flow) return null;
  const last = steps.done[steps.done.length - 1];
  const next = steps.undone[steps.undone.length - 1];
  const keep = (into: Flow, picked: FlowPicked) =>
    setChosen({ picked, sign: signOf(into, picked) });
  return {
    flow,
    picked: foundAgain(flow, chosen.picked, chosen.sign),
    change: (made) => {
      if (!made) return;
      const written = writeFlow({ ...flow, body: made.body, places: made.places ?? flow.places });
      if (written !== text) {
        setSteps({
          done: [...steps.done.slice(1 - KEPT_STEPS), { before: text, after: written }],
          undone: [],
        });
        setText(written);
      }
      keep(readFlow(written) ?? flow, made.picked);
    },
    pick: (picked) => keep(flow, picked),
    turn: (head) => setText(writeFlow({ ...flow, head })),
    undo:
      last?.after === text
        ? () => {
            setSteps({ done: steps.done.slice(0, -1), undone: [...steps.undone, last] });
            setText(last.before);
          }
        : null,
    redo:
      next?.before === text
        ? () => {
            setSteps({ done: [...steps.done, next], undone: steps.undone.slice(0, -1) });
            setText(next.after);
          }
        : null,
  };
};
