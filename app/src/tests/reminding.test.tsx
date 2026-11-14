import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { seenFrom, useCalledTasks, useWakeOffer } from "../reminding";

const bus = vi.hoisted(() => ({
  heard: new Map<string, (() => void)[]>(),
  waiting: null as string | null,
  tasks: {} as Record<string, unknown>,
  due: null as unknown,
  asked: [] as string[],
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: () => void) => {
    bus.heard.set(named, [...(bus.heard.get(named) ?? []), fn]);
    return Promise.resolve(() => {});
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { id?: string }) => {
    bus.asked.push(cmd);
    if (cmd === "task_waiting") {
      const owed = bus.waiting;
      bus.waiting = null;
      return Promise.resolve(owed);
    }
    if (cmd === "task_of") {
      const one = bus.tasks[args?.id ?? ""];
      return one ? Promise.resolve(one) : Promise.reject(new Error("notATaskId"));
    }
    if (cmd === "wake_due") return Promise.resolve(bus.due);
    return Promise.resolve(null);
  },
}));

const TASK = { id: "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T", title: "comprar pan", list: "L1" } as Task;

beforeEach(() => {
  bus.heard.clear();
  bus.waiting = null;
  bus.tasks = { [TASK.id]: TASK };
  bus.due = null;
  bus.asked = [];
});

function seeing() {
  return { setChosen: vi.fn(), setHeld: vi.fn(), setSelected: vi.fn() };
}

describe("a pressed reminder", () => {
  it("opens its task in its own list when Tisty was started by it", async () => {
    bus.waiting = TASK.id;
    const hands = seeing();

    renderHook(() => useCalledTasks(hands));

    await waitFor(() => expect(hands.setSelected).toHaveBeenCalledWith(TASK.id));
    expect(hands.setChosen).toHaveBeenCalledWith({ named: "lists", list: "L1" });
    expect(hands.setHeld).toHaveBeenCalledWith(TASK);
  });

  it("opens its task when Tisty was already running", async () => {
    const hands = seeing();
    renderHook(() => useCalledTasks(hands));
    await waitFor(() => expect(bus.heard.get("showtask")).toBeTruthy());

    bus.waiting = TASK.id;
    for (const fn of bus.heard.get("showtask") ?? []) fn();

    await waitFor(() => expect(hands.setSelected).toHaveBeenCalledWith(TASK.id));
  });

  it("whose task is gone opens nothing and says nothing", async () => {
    bus.waiting = "01KA7B3VJ9RZ0Q8M5N2X4Y6W1V";
    const hands = seeing();

    renderHook(() => useCalledTasks(hands));

    await waitFor(() => expect(bus.asked).toContain("task_of"));
    expect(hands.setSelected).not.toHaveBeenCalled();
  });

  it("of a task with no list opens it among every task", () => {
    expect(seenFrom({ ...TASK, list: undefined })).toEqual({ named: "tasks", slice: "all" });
  });
});

describe("the offer to start with the computer", () => {
  it("comes once the backend says it is due", async () => {
    bus.due = { offered: true, wakes: false, theirs: false };

    const { result } = renderHook(() => useWakeOffer({}));

    await waitFor(() => expect(result.current.waking).toEqual(bus.due));
  });

  it("put off stays away for the rest of the run", async () => {
    bus.due = { offered: true, wakes: false, theirs: false };
    const { result, rerender } = renderHook(({ looked }) => useWakeOffer(looked), {
      initialProps: { looked: {} as unknown },
    });
    await waitFor(() => expect(result.current.waking).not.toBeNull());

    result.current.settleWaking();
    rerender({ looked: {} });

    await waitFor(() => expect(result.current.waking).toBeNull());
    expect(bus.asked.filter((one) => one === "wake_due")).toHaveLength(1);
  });

  it("is not looked for before the first snapshot", () => {
    renderHook(() => useWakeOffer(undefined));

    expect(bus.asked).not.toContain("wake_due");
  });
});
