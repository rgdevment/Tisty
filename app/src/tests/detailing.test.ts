import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Snapshot, Task } from "../core";
import { detailOf, erasing, type Hands } from "../detailing";
import { fill } from "../locales";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];
let asked = true;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    calls.push({ cmd, args });
    return Promise.resolve({ id: String(args?.id ?? "t1"), title: "Hecha" });
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(asked),
}));

const task = {
  id: "t1",
  title: "Llamar",
  part_of: "w1",
  volume: { journal: 2 },
} as unknown as Task;

const hands = (): Hands & Record<string, ReturnType<typeof vi.fn>> =>
  ({
    data: { lists: [], tags: [{ tag: "casa" }], wholes: {} } as unknown as Snapshot,
    wholes: { w1: { title: "La mudanza" } } as unknown as Hands["wholes"],
    act: vi.fn((work: Promise<Task>) => void work.catch(() => {})),
    marking: vi.fn(),
    wipe: vi.fn(),
    shut: vi.fn(),
    close: vi.fn(),
    openDoc: vi.fn(),
    opening: vi.fn(),
    say: vi.fn(),
    fail: vi.fn(),
  }) as unknown as Hands & Record<string, ReturnType<typeof vi.fn>>;

const sent = () => calls.map((one) => one.cmd);
const flush = () => new Promise((ready) => setTimeout(ready, 0));

beforeEach(() => {
  calls.length = 0;
  asked = true;
});

describe("what the detail of a task is handed", () => {
  it("says what the task belongs to and what tags it can pick from", () => {
    const shown = detailOf(task, hands());

    expect(shown.partOf).toBe("La mudanza");
    expect(shown.known).toEqual(["casa"]);
  });

  it("sends every edit of the task through one way of acting", async () => {
    const h = hands();
    const shown = detailOf(task, h);

    shown.onPatch({ title: "Otra" } as never);
    shown.onStep("paso");
    shown.onMark("s1", true);
    shown.onDropStep("s1");
    shown.onLog("nota");
    shown.onReopen();
    shown.onStillOpen();
    shown.onFold(true);
    shown.onReadAs("story");
    shown.onOpenToAgents(true);
    shown.onHang(null);
    shown.onStepToPart("s1");
    shown.onAddPart("parte");
    await flush();

    expect(h.act).toHaveBeenCalledTimes(13);
    expect(sent()).toContain("task_of");
  });

  it("closes the detail when the task is completed or discarded", () => {
    const h = hands();
    const shown = detailOf(task, h);

    shown.onComplete();
    shown.onDiscard();

    expect(h.marking).toHaveBeenCalledWith("t1", "Llamar");
    expect(h.close).toHaveBeenCalledTimes(2);
  });

  it("says a part is done and opens another part", async () => {
    const h = hands();
    const shown = detailOf(task, h);

    shown.onCompletePart("p1", "La caja");
    await shown.onOpenPart("p2");

    expect(h.say).toHaveBeenCalledWith(fill("saidDone", "La caja"));
    expect(h.opening).toHaveBeenCalled();
  });

  it("hands erasing to the one who wipes", () => {
    const h = hands();

    detailOf(task, h).onErase();

    expect(h.wipe).toHaveBeenCalledWith(task);
  });
});

describe("erasing a task", () => {
  it("erases only after the person says yes", async () => {
    const gone = vi.fn();
    erasing(task, { clear: vi.fn(), gone, fail: vi.fn() });
    await flush();

    expect(sent()).toEqual(["erase"]);
    expect(gone).toHaveBeenCalled();
  });

  it("leaves the task alone when the person says no", async () => {
    asked = false;
    const gone = vi.fn();
    erasing(task, { clear: vi.fn(), gone, fail: vi.fn() });
    await flush();

    expect(sent()).toEqual([]);
    expect(gone).not.toHaveBeenCalled();
  });
});
