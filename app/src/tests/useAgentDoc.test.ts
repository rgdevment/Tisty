import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentDoc } from "../agentDocs";
import { useAgentDoc } from "../ui/useAgentDoc";

const ipc = vi.hoisted(() => ({
  answer: (_file: string): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (_cmd: string, args?: { file: string }) => ipc.answer(args?.file ?? ""),
}));

const RULE = "---\nagent: claude/rule/CLAUDE.md\n---\n# Rules\n";

const rule: AgentDoc = {
  kind: "rule",
  level: "claude",
  named: "Claude",
  name: "CLAUDE.md",
  code: false,
  folderOff: false,
  here: "applied",
  print: "p1",
  twin: false,
  places: [],
};

describe("an agent document's hold", () => {
  beforeEach(() => {
    ipc.answer = () => Promise.resolve(null);
  });

  it("never takes a stray agent property for a card", () => {
    const { result } = renderHook(() => useAgentDoc("notes.md", "---\nagent: yo\n---\n# Notes\n"));
    expect(result.current.held).toBe(false);
    expect(result.current.doc).toBeNull();
  });

  it("lets go when the core does not take the document for an agent's", async () => {
    const { result } = renderHook(() => useAgentDoc("odd.md", RULE));
    expect(result.current.held).toBe(true);
    await waitFor(() => expect(result.current.held).toBe(false));
    expect(result.current.doc).toBeNull();
  });

  it("lets go when asking fails", async () => {
    ipc.answer = () => Promise.reject(new Error("gone"));
    const { result } = renderHook(() => useAgentDoc("broken.md", RULE));
    await waitFor(() => expect(result.current.held).toBe(false));
  });

  it("holds a real rule until Edit, and a memory always", async () => {
    ipc.answer = () => Promise.resolve(rule);
    const { result } = renderHook(() => useAgentDoc("rule.md", RULE));
    await waitFor(() => expect(result.current.doc).not.toBeNull());
    expect(result.current.held).toBe(true);
    act(() => result.current.onEdit(true));
    expect(result.current.held).toBe(false);
    expect(result.current.editing).toBe(true);

    ipc.answer = () => Promise.resolve({ ...rule, kind: "memory" });
    const memory = renderHook(() => useAgentDoc("memory.md", RULE));
    await waitFor(() => expect(memory.result.current.doc).not.toBeNull());
    act(() => memory.result.current.onEdit(true));
    expect(memory.result.current.held).toBe(true);
  });

  it("keeps a card being written by hand open to its writer", async () => {
    ipc.answer = () => Promise.resolve(rule);
    const { result, rerender } = renderHook(({ body }) => useAgentDoc("new.md", body), {
      initialProps: { body: "# Rules\n" },
    });
    rerender({ body: RULE });
    expect(result.current.held).toBe(false);
    await waitFor(() => expect(result.current.doc).not.toBeNull());
    expect(result.current.held).toBe(false);
    expect(result.current.editing).toBe(true);
  });

  it("opens held again after the person went to another document", async () => {
    ipc.answer = () => Promise.resolve(rule);
    const { result, rerender } = renderHook(({ file }) => useAgentDoc(file, RULE), {
      initialProps: { file: "a.md" },
    });
    await waitFor(() => expect(result.current.doc).not.toBeNull());
    act(() => result.current.onEdit(true));
    rerender({ file: "b.md" });
    rerender({ file: "a.md" });
    expect(result.current.held).toBe(true);
  });
});
