import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Place } from "../agentDocs";
import { useAgentShelf } from "../ui/useAgentCard";

const place = { at: "/home/me/.claude/CLAUDE.md" } as Place;

describe("what an agent document's card keeps open", () => {
  it("drops an open list of backups once another document opens, and never brings it back", () => {
    const { result, rerender } = renderHook(({ file }) => useAgentShelf(file), {
      initialProps: { file: "a" as string | null },
    });
    act(() => result.current.setKept(place));
    expect(result.current.kept).toBe(place);
    rerender({ file: "b" });
    expect(result.current.kept).toBeNull();
    rerender({ file: "a" });
    expect(result.current.kept).toBeNull();
    expect(result.current.shown).toBeNull();
  });

  it("keeps nothing while no document is open", () => {
    const { result } = renderHook(() => useAgentShelf(null));
    act(() => result.current.setKept(place));
    expect(result.current.kept).toBeNull();
  });
});
