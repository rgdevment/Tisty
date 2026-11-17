import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";

const ipc = vi.hoisted(() => ({ owed: [] as string[], done: {} as Partial<Task> }));

vi.mock("../core", () => ({
  owed: () => Promise.resolve(ipc.owed),
  complete: (id: string) => Promise.resolve({ id, title: "x", ...ipc.done }),
}));

import { useMarking } from "../marking";

const finished = async (done: Partial<Task>, owed: string[] = []) => {
  ipc.done = done;
  ipc.owed = owed;
  const cheered: string[] = [];
  const { result } = renderHook(() =>
    useMarking({
      chosen: { named: "tasks" },
      act: () => {},
      say: () => {},
      setError: () => {},
      lookForAStar: () => {},
      cheer: (task) => cheered.push(task.id),
    }),
  );
  await act(async () => {
    result.current.marking("01T", "Vacuna");
  });
  await act(async () => {});
  return cheered;
};

describe("who gets a thumbs up when it is done", () => {
  it("cheers a whole task that does not come back", async () => {
    expect(await finished({})).toEqual(["01T"]);
  });

  it("does not cheer a part of another task, nor one that repeats", async () => {
    expect(await finished({ part_of: "01W" })).toEqual([]);
    expect(await finished({ repeat: {} as unknown as Task["repeat"] })).toEqual([]);
  });

  it("does not cheer a routine that still owes days, which asks first", async () => {
    expect(await finished({}, ["2026-10-14"])).toEqual([]);
  });
});

describe("a routine finished from elsewhere", () => {
  it("keeps its strip of missed days in the view it is taken to, and drops it on the next move", async () => {
    const hands = {
      act: () => {},
      say: () => {},
      setError: () => {},
      lookForAStar: () => {},
    };
    const there = { named: "lists" as const, list: "01L" };
    const { result, rerender } = renderHook(({ chosen }) => useMarking({ ...hands, chosen }), {
      initialProps: { chosen: { named: "docs" } as Parameters<typeof useMarking>[0]["chosen"] },
    });
    await act(async () => {
      result.current.owing("01T", "Pasear a Roger", ["2026-11-10"], there);
      rerender({ chosen: there });
    });
    expect(result.current.asking?.days).toEqual(["2026-11-10"]);
    expect(result.current.strip).not.toBeNull();

    rerender({ chosen: { named: "tasks" } });
    expect(result.current.asking).toBeNull();
  });
});
