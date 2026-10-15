import { act, render, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";

const ipc = vi.hoisted(() => ({ quiet: [] as string[], fails: false }));

vi.mock("../core", () => ({
  settings: () =>
    ipc.fails ? Promise.reject(new Error("no")) : Promise.resolve({ quiet: ipc.quiet }),
}));

import { Head, Paws, QUIET, Thumb, useBeaver } from "../ui/Beaver";
import Cheer from "../ui/Cheer";

const task = { id: "01T", title: "Vacuna de Rina" } as Task;

beforeEach(() => {
  ipc.quiet = [];
  ipc.fails = false;
});

describe("Tisto, the beaver that peeks in", () => {
  it("comes unless the person turned it off, and stays away when settings cannot be read", async () => {
    const on = renderHook(() => useBeaver());
    await waitFor(() => expect(on.result.current).toBe(true));
    ipc.quiet = [QUIET];
    const off = renderHook(() => useBeaver());
    await waitFor(() => expect(off.result.current).toBe(false));
    ipc.quiet = [];
    ipc.fails = true;
    const unknown = renderHook(() => useBeaver());
    await act(async () => {});
    expect(unknown.result.current).toBe(false);
  });

  it("reads the settings again when the window comes back, so turning it off reaches every window", async () => {
    const seen = renderHook(() => useBeaver());
    await waitFor(() => expect(seen.result.current).toBe(true));
    ipc.quiet = [QUIET];
    window.dispatchEvent(new Event("focus"));
    await waitFor(() => expect(seen.result.current).toBe(false));
  });

  it("draws its head, its paws with three fingers each, and its thumb", () => {
    const view = render(
      <>
        <Head className="one" />
        <Paws />
        <Thumb className="two" />
      </>,
    );
    expect(view.container.querySelector("svg.one")).not.toBeNull();
    expect(view.container.querySelectorAll(".paws .paw")).toHaveLength(2);
    expect(view.container.querySelectorAll(".paw i")).toHaveLength(6);
    expect(view.container.querySelector("svg.two")).not.toBeNull();
  });
});

describe("the thumbs up when a task is done", () => {
  it("says the task is done and kept, undoes it, and leaves on its own", () => {
    const undone: string[] = [];
    let left = 0;
    const view = render(
      <Cheer
        task={task}
        onUndo={() => undone.push(task.id)}
        onDismiss={() => {
          left += 1;
        }}
      />,
    );
    expect(view.container.textContent).toContain("Vacuna de Rina");
    view.getByRole("button").click();
    expect(undone).toEqual(["01T"]);
    act(() => {
      vi.advanceTimersByTime(5100);
    });
    expect(left).toBe(1);
  });
});
