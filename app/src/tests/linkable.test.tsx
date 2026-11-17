import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { useLinkable } from "../ui/useLinkable";

const asked = vi.hoisted(() => ({ heads: 0 }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd !== "doc_heads") return Promise.resolve(null);
    asked.heads += 1;
    return Promise.resolve([
      { id: "hoteles", title: "Hoteles", level: 2 },
      { id: "comida", title: "Comida", level: 2 },
    ]);
  },
}));

const doc = (file: string, title: string, wrote: string, more: Partial<Filed> = {}): Filed =>
  ({
    id: file,
    file,
    title,
    folder: null,
    archived: false,
    away: false,
    wrote,
    ...more,
  }) as Filed;

const KNOWN = [
  doc("a", "Viajes al sur", "2026-06-01T00:00:00Z"),
  doc("b", "Mis viajes", "2026-06-09T00:00:00Z"),
  doc("c", "Viajes", "2026-06-05T00:00:00Z"),
  doc("d", "Viajes viejos", "2026-06-08T00:00:00Z", { away: true }),
  doc("e", "Abierto", "2026-06-10T00:00:00Z"),
];

describe("documents offered after [[", () => {
  it("ranks titles that begin with the words first, and leaves out the open and the put away", () => {
    const { result } = renderHook(() => useLinkable(KNOWN, "e"));

    expect(result.current.linkable("viajes").map((one) => one.label)).toEqual([
      "Viajes",
      "Viajes al sur",
      "Mis viajes",
    ]);
    expect(result.current.linkable("").map((one) => one.key)).toEqual(["b", "c", "a"]);
    expect(result.current.linkable("viajes")[0].link).toBe("[Viajes](tisty:doc/c)");
  });

  it("brings a document's headings once, then narrows them by what follows #", async () => {
    const { result } = renderHook(() => useLinkable(KNOWN, "e"));
    const ready = vi.fn();

    expect(result.current.headsOf("Viajes", "", ready)).toEqual([]);
    await waitFor(() => expect(ready).toHaveBeenCalledTimes(1));
    const heads = result.current.headsOf("Viajes", "hot", ready);

    expect(heads.map((one) => [one.label, one.link])).toEqual([
      ["Hoteles", "[Viajes › Hoteles](tisty:doc/c#hoteles)"],
    ]);
    expect(asked.heads).toBe(1);
  });
});
