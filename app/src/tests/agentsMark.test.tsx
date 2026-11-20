import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsState } from "../agentsSync";
import type { Folded, Papers } from "../core";
import { AGENTS_CHANGED } from "../ui/AgentsKeeper";
import { useAgentsStage, useAgentsState } from "../ui/AgentsMark";

const ipc = vi.hoisted(() => ({
  asks: 0,
  answer: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => {
    ipc.asks += 1;
    return ipc.answer();
  },
}));

const state = (more: Partial<AgentsState> = {}): AgentsState => ({
  folder: "A",
  held: 0,
  consented: true,
  allOn: true,
  assistants: [],
  ...more,
});

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
  ...more,
});

const withAgents = (docs = 1): Papers => ({
  folders: [folder("A", "AGENTS", { icon: "robot" })],
  docs: Array.from({ length: docs }, (_, at) => ({
    id: `D${at}`,
    file: `f${at}`,
    title: `Regla ${at}`,
    folder: "A",
    archived: false,
    away: false,
  })),
});

beforeEach(() => {
  ipc.asks = 0;
  ipc.answer = () => Promise.resolve(state());
});

describe("what this computer says about Agents", () => {
  it("asks nothing while there is no folder", () => {
    const { result } = renderHook(() => useAgentsState(null));
    expect(result.current).toBeNull();
    expect(ipc.asks).toBe(0);
  });

  it("asks again after a round, and keeps only the latest answer", async () => {
    const slow: { give?: (one: AgentsState) => void } = {};
    ipc.answer = () => new Promise((give) => (slow.give = give));
    const { result } = renderHook(() => useAgentsState("A"));
    const first = slow.give;
    ipc.answer = () => Promise.resolve(state({ allOn: false }));
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await waitFor(() => expect(result.current?.allOn).toBe(false));
    await act(async () => first?.(state()));
    expect(result.current?.allOn).toBe(false);
  });

  it("forgets a failed answer", async () => {
    const { result } = renderHook(() => useAgentsState("A"));
    await waitFor(() => expect(result.current).not.toBeNull());
    ipc.answer = () => Promise.reject(new Error("gone"));
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await waitFor(() => expect(result.current).toBeNull());
  });
});

describe("the stage the sidebar draws", () => {
  it("is unbrought with no folder, without asking", () => {
    const { result } = renderHook(() => useAgentsStage({ folders: [], docs: [] }));
    expect(result.current).toEqual({ root: null, stance: null, stage: "unbrought" });
    expect(ipc.asks).toBe(0);
  });

  it("is unknown until the answer comes, then syncs", async () => {
    const { result } = renderHook(() => useAgentsStage(withAgents()));
    expect(result.current.stage).toBeNull();
    await waitFor(() => expect(result.current.stage).toBe("syncs"));
    expect(result.current.root).toBe("A");
  });

  it("is brought when the folder holds documents and does not sync", async () => {
    ipc.answer = () => Promise.resolve(state({ consented: false, allOn: false, held: 2 }));
    const { result } = renderHook(() => useAgentsStage(withAgents(2)));
    await waitFor(() => expect(result.current.stage).toBe("brought"));
  });

  it("is unbrought when the empty folder was never allowed", async () => {
    ipc.answer = () => Promise.resolve(state({ consented: false }));
    const { result } = renderHook(() => useAgentsStage(withAgents(0)));
    await waitFor(() => expect(result.current.stance).not.toBeNull());
    expect(result.current.stage).toBe("unbrought");
  });
});
