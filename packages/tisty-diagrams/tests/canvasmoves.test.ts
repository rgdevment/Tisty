import { act, renderHook } from "@testing-library/react";
import type { NodeChange } from "@xyflow/react";
import { describe, expect, it, vi } from "vitest";
import { type Flow, readFlow, writeFlow } from "../src/flow";
import type { FlowAltered, FlowPicked } from "../src/flowing";
import { flowLaid, lanesOf } from "../src/scenes";
import { useCanvasMoves } from "../src/useCanvasMoves";
import type { Flowing } from "../src/useFlow";

const SOURCE =
  "flowchart LR\n  subgraph cli [Cliente]\n    A[Pide]\n  end\n  A --> B\n  C\n  %% tisty: cli 100 0 260 480\n  %% tisty: A 140 60\n  %% tisty: B 400 60\n  %% tisty: C 400 200";

const moving = (picked: FlowPicked = null) => {
  const flow = readFlow(SOURCE) as Flow;
  const change = vi.fn<(made: FlowAltered | null) => void>();
  const pick = vi.fn<(picked: FlowPicked) => void>();
  const editing: Flowing = {
    flow,
    picked,
    change,
    pick,
    undo: null,
    redo: null,
  };
  const lanes = new Map(lanesOf(flow).map((one) => [one.id, one]));
  const hook = renderHook(() => useCanvasMoves(editing, flowLaid(flow), lanes));
  const text = () => {
    const made = change.mock.lastCall?.[0];
    return made ? writeFlow({ ...flow, body: made.body, places: made.places ?? flow.places }) : "";
  };
  return { hook, change, pick, text };
};

describe("moving and sizing on the canvas", () => {
  it("draws a lane as it is sized, and writes its size only once it is let go", () => {
    const { hook, change, text } = moving();
    const sizedTo = (width: number, height: number): NodeChange => ({
      type: "dimensions",
      id: "group:cli",
      dimensions: { width, height },
      resizing: true,
    });

    act(() => hook.result.current.sizing());
    act(() => hook.result.current.moved([sizedTo(300, 500)]));
    act(() =>
      hook.result.current.moved([
        { type: "position", id: "group:cli", position: { x: -10, y: 0 } },
      ]),
    );
    expect(hook.result.current.moving.get("group:cli")).toEqual({
      x: -10,
      y: 0,
      wide: 300,
      tall: 500,
    });
    expect(change).not.toHaveBeenCalled();

    act(() => hook.result.current.sized("cli", { x: -10, y: 0, wide: 310, tall: 500 }));
    expect(text()).toContain("%% tisty: cli -10 0 310 500");
    expect(hook.result.current.moving.size).toBe(0);
  });

  it("writes a shape dragged inside its lane where it ends, the lane's corner added back", () => {
    const { hook, text } = moving();

    act(() =>
      hook.result.current.moved([
        { type: "position", id: "A", position: { x: 20, y: 20 }, dragging: true },
      ]),
    );
    expect(hook.result.current.laid.get("A")).toMatchObject({ x: 120, y: 20 });
    act(() =>
      hook.result.current.moved([
        { type: "position", id: "A", position: { x: 30, y: 25 }, dragging: false },
      ]),
    );

    expect(text()).toContain("%% tisty: A 130 25");
  });

  it("chooses a shape chosen alone, and several chosen together", () => {
    const several = moving({ kind: "nodes", ids: ["B", "C"] });
    act(() =>
      several.hook.result.current.moved([
        { type: "select", id: "A", selected: true },
        { type: "select", id: "B", selected: false },
        { type: "select", id: "C", selected: false },
      ]),
    );
    expect(several.pick).toHaveBeenLastCalledWith({ kind: "node", id: "A" });

    const one = moving({ kind: "node", id: "B" });
    act(() => one.hook.result.current.moved([{ type: "select", id: "C", selected: true }]));
    expect(one.pick).toHaveBeenLastCalledWith({ kind: "nodes", ids: ["B", "C"] });

    act(() => one.hook.result.current.moved([{ type: "select", id: "group:cli", selected: true }]));
    expect(one.pick).toHaveBeenCalledTimes(1);
  });
});
