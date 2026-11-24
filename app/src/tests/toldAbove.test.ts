import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import * as core from "../core";
import { useToldAbove } from "../ui/useToldAbove";

afterEach(() => vi.restoreAllMocks());

it("wears the design of the document above, and never the one of the document before", async () => {
  const read = vi.spyOn(core, "docRead");
  read.mockResolvedValueOnce("---\ndesign: coral\n---\n# Uno");
  const { result, rerender } = renderHook(({ book }) => useToldAbove(book, 0), {
    initialProps: { book: "uno" as string | undefined },
  });
  await waitFor(() => expect(result.current.design).toBe("coral"));
  read.mockReturnValueOnce(new Promise(() => {}));
  rerender({ book: "dos" });
  expect(result.current.design).toBeUndefined();
});
