import { act, render, renderHook, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Glance } from "../agentDocs";
import { AGENTS_CHANGED } from "../ui/AgentsKeeper";
import { AgentsWord, useAgentsGlance } from "../ui/AgentsMark";
import { AGENT_DOC_STOOD } from "../ui/useAgentDoc";

const ipc = vi.hoisted(() => ({
  looks: 0,
  answer: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => {
    ipc.looks += 1;
    return ipc.answer();
  },
}));

const glance = (more: Partial<Glance> = {}): Glance => ({
  folder: "a",
  docs: 3,
  changes: 0,
  waiting: 0,
  unagreed: 0,
  proposed: 0,
  consented: true,
  allOn: true,
  ...more,
});

beforeEach(() => {
  ipc.looks = 0;
  ipc.answer = () => Promise.resolve(glance());
});

describe("the AGENTS word in the sidebar", () => {
  it("looks only once AGENTS exists", () => {
    const { result } = renderHook(() => useAgentsGlance(null));
    expect(result.current).toBeNull();
    expect(ipc.looks).toBe(0);
  });

  it("looks again after a round, a card's news and the window's return", async () => {
    const { result } = renderHook(() => useAgentsGlance("a"));
    await waitFor(() => expect(result.current?.docs).toBe(3));
    ipc.answer = () => Promise.resolve(glance({ changes: 2 }));
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await waitFor(() => expect(result.current?.changes).toBe(2));
    act(() => window.dispatchEvent(new Event(AGENT_DOC_STOOD)));
    act(() => window.dispatchEvent(new Event("focus")));
    await waitFor(() => expect(ipc.looks).toBe(4));
  });

  it("keeps only the latest answer and forgets a failed look", async () => {
    const slow: { give?: (one: Glance) => void } = {};
    ipc.answer = () => new Promise((give) => (slow.give = give));
    const { result } = renderHook(() => useAgentsGlance("a"));
    const first = slow.give;
    ipc.answer = () => Promise.resolve(glance({ changes: 1 }));
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await waitFor(() => expect(result.current?.changes).toBe(1));
    await act(async () => first?.(glance({ changes: 9 })));
    expect(result.current?.changes).toBe(1);
    ipc.answer = () => Promise.reject(new Error("gone"));
    act(() => window.dispatchEvent(new Event("focus")));
    await waitFor(() => expect(result.current).toBeNull());
  });

  it("says empty, in sync, how many changed, or off", () => {
    const { rerender } = render(<AgentsWord glance={glance({ docs: 0 })} />);
    expect(screen.getByText("empty")).toBeTruthy();
    rerender(<AgentsWord glance={glance()} />);
    expect(screen.getByText("in sync")).toBeTruthy();
    rerender(<AgentsWord glance={glance({ changes: 2 })} />);
    expect(screen.getByText("2 changed")).toBeTruthy();
    rerender(<AgentsWord glance={glance({ waiting: 3, unagreed: 1 })} />);
    expect(screen.getByText("3 waiting")).toBeTruthy();
    rerender(<AgentsWord glance={glance({ unagreed: 1 })} />);
    expect(screen.getByText("1 to approve")).toBeTruthy();
    rerender(<AgentsWord glance={glance({ proposed: 2 })} />);
    expect(screen.getByText("2 with a proposal")).toBeTruthy();
    rerender(<AgentsWord glance={glance({ allOn: false })} />);
    expect(screen.getByText("off")).toBeTruthy();
  });
});
