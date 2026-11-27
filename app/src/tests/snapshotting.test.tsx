import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useSnapshot } from "../snapshotting";

const core = vi.hoisted(() => ({ title: "write the report" }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "settle_in") return Promise.resolve({ ran: false, brought: false, agrees: true });
    if (cmd !== "snapshot") return Promise.resolve(null);
    return Promise.resolve({
      tasks: [{ id: "01A", title: core.title }],
      total: 1,
      lists: [],
      locale: "en",
      agents: {},
    });
  },
}));

beforeEach(() => {
  core.title = "write the report";
});

const opened = async () => {
  const hands = {
    seen: { named: "tasks" } as const,
    reach: 20,
    acted: { current: null },
    setError: vi.fn(),
  };
  const hook = renderHook(() => useSnapshot(hands));
  await waitFor(() => expect(hook.result.current.data).not.toBeNull());
  return hook;
};

describe("the snapshot the window holds", () => {
  it("keeps the same object when the core answers the same again", async () => {
    const hook = await opened();
    const first = hook.result.current.data;

    await act(async () => hook.result.current.load());

    expect(hook.result.current.data).toBe(first);
  });

  it("holds the new object as soon as the answer differs", async () => {
    const hook = await opened();
    const first = hook.result.current.data;

    core.title = "write the other report";
    await act(async () => hook.result.current.load());

    await waitFor(() => expect(hook.result.current.data).not.toBe(first));
    expect(hook.result.current.data?.tasks[0].title).toBe("write the other report");
  });
});
