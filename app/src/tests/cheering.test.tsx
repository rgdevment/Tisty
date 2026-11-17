import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";

const ipc = vi.hoisted(() => ({ owed: [] as string[], done: {} as Partial<Task>, asked: 0 }));

vi.mock("../core", () => ({
  owed: () => {
    ipc.asked += 1;
    return Promise.resolve(ipc.owed);
  },
  complete: (id: string) => Promise.resolve({ id, title: "x", ...ipc.done }),
  taskOf: (id: string) => Promise.resolve({ id, title: "x", list: "01L" }),
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

describe("a task finished from a card elsewhere", () => {
  const hands = { act: () => {}, say: () => {}, setError: () => {}, lookForAStar: () => {} };
  type Chosen = Parameters<typeof useMarking>[0]["chosen"];

  it("takes a routine that owes days to its list and keeps the strip there, until the next move", async () => {
    ipc.owed = ["2026-11-10"];
    ipc.asked = 0;
    const there: Chosen = { named: "lists", list: "01L" };
    const { result, rerender } = renderHook(({ chosen }) => useMarking({ ...hands, chosen }), {
      initialProps: { chosen: { named: "docs" } as Chosen },
    });
    await act(async () => {
      result.current.markingAway("01T", "Pasear a Roger", () => {
        rerender({ chosen: there });
        return there;
      });
    });
    await act(async () => {});
    expect(result.current.asking?.days).toEqual(["2026-11-10"]);
    expect(result.current.strip).not.toBeNull();
    expect(ipc.asked).toBe(1);

    rerender({ chosen: { named: "tasks" } });
    expect(result.current.asking).toBeNull();
  });

  it("finishes a task that owes nothing where it is, asking once", async () => {
    ipc.owed = [];
    ipc.asked = 0;
    ipc.done = {};
    const went = vi.fn();
    const cheered: string[] = [];
    const { result } = renderHook(() =>
      useMarking({ ...hands, chosen: { named: "docs" }, cheer: (task) => cheered.push(task.id) }),
    );
    await act(async () => {
      result.current.markingAway("01T", "Vacuna", went);
    });
    await act(async () => {});
    expect(went).not.toHaveBeenCalled();
    expect(cheered).toEqual(["01T"]);
    expect(ipc.asked).toBe(1);
  });
});
