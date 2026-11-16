import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useRestrips } from "../ui/useRestrips";

const ipc = vi.hoisted(() => ({ said: "", asked: [] as { file: string; body: string }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (_cmd: string, args: { file: string; body: string }) => {
    ipc.asked.push(args);
    return Promise.resolve(ipc.said);
  },
}));

beforeEach(() => {
  ipc.said = "15:true:";
  ipc.asked = [];
});

describe("redrawing a diary page for its strips", () => {
  it("draws again only when a later round says something new about the same page", async () => {
    const { result, rerender } = renderHook(
      ({ file, round }) => useRestrips(file, "# Noviembre", round),
      { initialProps: { file: "a3f1-0011" as string | undefined, round: 0 } },
    );
    await waitFor(() => expect(ipc.asked).toHaveLength(1));
    expect(result.current).toBe(0);

    rerender({ file: "a3f1-0011", round: 1 });
    await waitFor(() => expect(ipc.asked).toHaveLength(2));
    expect(result.current).toBe(0);

    ipc.said = "15:true:01T";
    rerender({ file: "a3f1-0011", round: 2 });
    await waitFor(() => expect(result.current).toBe(1));

    ipc.said = "3:false:01X";
    rerender({ file: "a3f1-0012", round: 2 });
    await waitFor(() => expect(ipc.asked).toHaveLength(4));
    expect(result.current).toBe(1);
    expect(ipc.asked[3]).toEqual({ file: "a3f1-0012", body: "# Noviembre" });
  });

  it("asks nothing for a page outside the diary", () => {
    renderHook(() => useRestrips(undefined, "# Notas", 3));

    expect(ipc.asked).toEqual([]);
  });
});
