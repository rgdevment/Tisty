import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Place } from "../agentDocs";
import { useAgentShelf } from "../ui/useAgentCard";

const place = { at: "/home/me/.claude/CLAUDE.md" } as Place;
const shown = { place, text: "Tests first.\n" };

describe("what an agent document's card keeps open", () => {
  it("drops the file shown once another document opens, and never brings it back", () => {
    const { result, rerender } = renderHook(({ file }) => useAgentShelf(file), {
      initialProps: { file: "a" as string | null },
    });
    act(() => result.current.setShown(shown));
    expect(result.current.shown).toBe(shown);
    rerender({ file: "b" });
    expect(result.current.shown).toBeNull();
    rerender({ file: "a" });
    expect(result.current.shown).toBeNull();
  });

  it("keeps nothing while no document is open", () => {
    const { result } = renderHook(() => useAgentShelf(null));
    act(() => result.current.setShown(shown));
    expect(result.current.shown).toBeNull();
  });
});
