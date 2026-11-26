import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const host = vi.hoisted(() => ({
  heard: null as (() => void) | null,
  name: "",
  invoked: [] as string[],
  off: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, then: () => void) => {
    host.name = name;
    host.heard = then;
    return Promise.resolve(host.off);
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    host.invoked.push(cmd);
    return Promise.resolve(null);
  },
}));

import { PASTE_PLAINLY, usePlainPasting } from "../ui/plainPasting";

beforeEach(() => {
  host.heard = null;
  host.name = "";
  host.invoked = [];
  host.off.mockClear();
});

describe("the menu's paste without formatting", () => {
  it("listens to its own event", async () => {
    renderHook(() => usePlainPasting(() => null));
    await waitFor(() => expect(host.heard).not.toBeNull());
    expect(host.name).toBe(PASTE_PLAINLY);
    expect(PASTE_PLAINLY).toBe("paste-plainly");
  });

  it("asks the focused page for it and pastes through the host", async () => {
    const plainNext = vi.fn();
    renderHook(() => usePlainPasting(() => ({ plainNext })));
    await waitFor(() => expect(host.heard).not.toBeNull());
    host.heard?.();
    expect(plainNext).toHaveBeenCalledTimes(1);
    expect(host.invoked).toEqual(["pasted_plainly"]);
  });

  it("does neither when no page has the focus", async () => {
    renderHook(() => usePlainPasting(() => null));
    await waitFor(() => expect(host.heard).not.toBeNull());
    host.heard?.();
    expect(host.invoked).toEqual([]);
  });

  it("asks the page that has the focus when the event comes, not when it was set up", async () => {
    const first = vi.fn();
    const second = vi.fn();
    let page: { plainNext: () => void } | null = { plainNext: first };
    renderHook(() => usePlainPasting(() => page));
    await waitFor(() => expect(host.heard).not.toBeNull());
    page = { plainNext: second };
    host.heard?.();
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledTimes(1);
  });

  it("lets go of the listener when its window goes", async () => {
    const view = renderHook(() => usePlainPasting(() => null));
    await waitFor(() => expect(host.heard).not.toBeNull());
    view.unmount();
    await waitFor(() => expect(host.off).toHaveBeenCalledTimes(1));
  });
});
