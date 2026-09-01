import { describe, expect, it } from "vitest";
import type { Folded } from "../core";
import { trailed } from "../ui/Beside";

const folder = (id: string, name: string, parent: string | null): Folded => ({
  id,
  name,
  parent,
  icon: null,
  holds: 0,
});

describe("the path that says where a document is kept", () => {
  const all = [
    folder("a", "Personal", null),
    folder("b", "Proyectos", "a"),
    folder("c", "Tisty", "b"),
    folder("d", "Otra", null),
  ];

  it("walks from the root down to the folder it is in", () => {
    expect(trailed(all, "c").map((one) => one.name)).toEqual(["Personal", "Proyectos", "Tisty"]);
  });

  it("is empty for a document in no folder", () => {
    expect(trailed(all, null)).toEqual([]);
  });

  it("is empty when the folder is not among the ones it was given", () => {
    expect(trailed(all, "gone")).toEqual([]);
  });

  it("stops instead of looping when a folder is its own ancestor", () => {
    const bent = [folder("x", "Una", "y"), folder("y", "Otra", "x")];
    expect(trailed(bent, "x")).toHaveLength(2);
  });

  it("stops at the first folder it cannot find, keeping what it walked", () => {
    expect(trailed([folder("c", "Tisty", "missing")], "c").map((one) => one.name)).toEqual([
      "Tisty",
    ]);
  });
});
