import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Papers } from "../core";
import { usePinned } from "../usePinned";

const ipc = vi.hoisted(() => ({ asked: 0, ideas: 3 }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd !== "brainstorm_of") return Promise.resolve(null);
    ipc.asked += 1;
    return Promise.resolve({ file: "ideas-0001", ideas: ipc.ideas, marked: 0 });
  },
}));

const doc = (file: string, over: Partial<Filed> = {}): Filed => ({
  id: file,
  file,
  title: file,
  folder: null,
  archived: false,
  away: false,
  wrote: "2026-11-26T10:00:00Z",
  bytes: 10,
  ...over,
});

const papersOf = (...docs: Filed[]): Papers => ({ folders: [], docs });

const settled = async (result: { current: unknown }, asked: number) => {
  await waitFor(() => expect(result.current).not.toBeNull());
  await waitFor(() => expect(ipc.asked).toBe(asked));
};

describe("usePinned", () => {
  beforeEach(() => {
    ipc.asked = 0;
    ipc.ideas = 3;
  });

  it("asks once on the first render and returns the answer", async () => {
    const { result } = renderHook(() => usePinned(papersOf(doc("a"))));
    await waitFor(() =>
      expect(result.current).toEqual({ file: "ideas-0001", ideas: 3, marked: 0 }),
    );
    expect(ipc.asked).toBe(1);
  });

  it("does not ask again when only another document's words or bytes changed", async () => {
    const sheet = doc("ideas-0001");
    const { result, rerender } = renderHook(({ papers }) => usePinned(papers), {
      initialProps: { papers: papersOf(doc("a"), sheet) },
    });
    await settled(result, 2);
    const before = ipc.asked;
    rerender({ papers: papersOf(doc("a", { wrote: "2026-11-26T10:00:02Z", bytes: 99 }), sheet) });
    await new Promise((done) => setTimeout(done, 20));
    expect(ipc.asked).toBe(before);
  });

  it("asks again when a document is added", async () => {
    const { result, rerender } = renderHook(({ papers }) => usePinned(papers), {
      initialProps: { papers: papersOf(doc("a")) },
    });
    await settled(result, 1);
    rerender({ papers: papersOf(doc("a"), doc("b")) });
    await waitFor(() => expect(ipc.asked).toBe(2));
  });

  it("asks again when a document is removed", async () => {
    const { result, rerender } = renderHook(({ papers }) => usePinned(papers), {
      initialProps: { papers: papersOf(doc("a"), doc("b")) },
    });
    await settled(result, 1);
    rerender({ papers: papersOf(doc("a")) });
    await waitFor(() => expect(ipc.asked).toBe(2));
  });

  it("asks again when a document is archived", async () => {
    const { result, rerender } = renderHook(({ papers }) => usePinned(papers), {
      initialProps: { papers: papersOf(doc("a")) },
    });
    await settled(result, 1);
    rerender({ papers: papersOf(doc("a", { archived: true })) });
    await waitFor(() => expect(ipc.asked).toBe(2));
  });

  it("asks again and updates the counts when the Ideas sheet's own entry changes", async () => {
    const { result, rerender } = renderHook(({ papers }) => usePinned(papers), {
      initialProps: { papers: papersOf(doc("a"), doc("ideas-0001")) },
    });
    await settled(result, 2);
    ipc.ideas = 5;
    rerender({ papers: papersOf(doc("a"), doc("ideas-0001", { wrote: "2026-11-26T11:00:00Z" })) });
    await waitFor(() => expect(result.current?.ideas).toBe(5));
    const afterWrote = ipc.asked;
    ipc.ideas = 6;
    rerender({
      papers: papersOf(
        doc("a"),
        doc("ideas-0001", {
          wrote: "2026-11-26T11:00:00Z",
          ideasFlagged: [{ heading: "h", text: "t" } as never],
        }),
      ),
    });
    await waitFor(() => expect(result.current?.ideas).toBe(6));
    expect(ipc.asked).toBeGreaterThan(afterWrote);
  });
});
