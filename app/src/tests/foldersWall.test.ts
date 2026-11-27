import { afterEach, describe, expect, it, vi } from "vitest";
import type { Folded } from "../core";
import { destinations, wallFor } from "../folders";
import { adopt } from "../locales";
import { saidPlainly } from "../refusal";
import { fits, settled } from "../ui/dragging";

const folder = (id: string, name: string, parent: string | null, icon: string | null = null) =>
  ({ id, name, parent, icon, holds: 0, archived: false, away: false }) as Folded;

const folders: Folded[] = [
  folder("AG", "AGENTS", null, "robot"),
  folder("AG1", "Global", "AG"),
  folder("AG2", "Claude", "AG"),
  folder("AG21", "Deep", "AG2"),
  folder("P1", "work", null),
  folder("P2", "sub", "P1"),
  folder("P3", "home", null),
];

const keys = (list: ReturnType<typeof destinations>) => list.map((one) => one.key);
const byId = (id: string) => folders.find((one) => one.id === id) as Folded;

describe("wallFor", () => {
  it("walls the whole Agents tree off a plain folder", () => {
    expect([...wallFor(folders, "P1")].sort()).toEqual(["AG", "AG1", "AG2", "AG21"]);
    expect([...wallFor(folders, "P2")].sort()).toEqual(["AG", "AG1", "AG2", "AG21"]);
  });

  it("walls every folder outside Agents off one of Agents", () => {
    expect([...wallFor(folders, "AG1")].sort()).toEqual(["P1", "P2", "P3"]);
    expect([...wallFor(folders, "AG21")].sort()).toEqual(["P1", "P2", "P3"]);
  });

  it("walls nothing when there is no Agents folder", () => {
    expect(
      wallFor(
        folders.filter((one) => one.id.startsWith("P")),
        "P1",
      ).size,
    ).toBe(0);
  });
});

describe("destinations with the walls", () => {
  const land = vi.fn();

  it("keeps a plain folder out of Agents and leaves Unfiled on", () => {
    const list = destinations(folders, "P1", land, byId("P2"));
    expect(keys(list)).toEqual(["unfiled", "P3"]);
    expect(list[0].off).toBe(false);
  });

  it("lets a subfolder of Agents choose only Agents folders, with Unfiled off", () => {
    const list = destinations(folders, "AG2", land, byId("AG21"));
    expect(keys(list)).toEqual(["unfiled", "AG", "AG1"]);
    expect(list[0].off).toBe(true);
  });

  it("leaves Unfiled on for the Agents root itself", () => {
    const list = destinations(folders, "P1", land, byId("AG"));
    expect(list[0].off).toBe(false);
    expect(keys(list)).toEqual(["unfiled", "P3", "P2"].filter((k) => keys(list).includes(k)));
    expect(keys(list)).not.toContain("AG1");
  });

  it("still excludes itself and its descendants", () => {
    const list = destinations(folders, null, land, byId("P1"));
    expect(keys(list)).toEqual(["unfiled", "P3"]);
  });

  it("still honours the depth limit", () => {
    const deepFolders = [
      folder("A", "a", null),
      folder("B", "b", "A"),
      folder("C", "c", "B"),
      folder("D", "d", "C"),
      folder("E", "e", "D"),
      folder("X", "x", null),
      folder("Y", "y", "X"),
    ];
    const list = destinations(deepFolders, null, land, deepFolders[5]);
    expect(keys(list)).not.toContain("E");
    expect(keys(list)).toContain("D");
  });

  it("keeps Unfiled off without a mover when already loose", () => {
    expect(destinations(folders, null, land)[0].off).toBe(true);
  });
});

describe("dragging with the walls", () => {
  const into = { id: "P1", kind: "folder" as const, holds: true, line: [], depth: 0 };

  it("refuses an Agents subfolder dropped into a plain folder", () => {
    const carried = {
      id: "AG1",
      kind: "folder" as const,
      tall: 1,
      walled: wallFor(folders, "AG1"),
    };
    expect(fits(carried, into, "in")).toBe(false);
    expect(settled(carried, into, "in")).toBeNull();
  });

  it("refuses a plain folder dropped into Agents", () => {
    const carried = { id: "P3", kind: "folder" as const, tall: 1, walled: wallFor(folders, "P3") };
    const agents = { ...into, id: "AG1", line: ["AG"], depth: 1 };
    expect(settled(carried, agents, "in")).toBeNull();
  });

  it("lets an Agents subfolder land in another Agents folder", () => {
    const carried = {
      id: "AG1",
      kind: "folder" as const,
      tall: 1,
      walled: wallFor(folders, "AG1"),
    };
    const agents = { ...into, id: "AG2", line: ["AG"], depth: 1 };
    expect(settled(carried, agents, "in")).toEqual({ kind: "folder", moved: "AG1", folder: "AG2" });
  });
});

describe("agentsKeepTheirFolders", () => {
  afterEach(() => adopt("en"));

  it("has an English sentence, not the raw code", () => {
    adopt("en");
    const said = saidPlainly({ code: "agentsKeepTheirFolders" });
    expect(said).toBe("A folder of Agents stays inside Agents, where its agent reads it.");
  });

  it("has a Spanish sentence, not the raw code", () => {
    adopt("es");
    const said = saidPlainly({ code: "agentsKeepTheirFolders" });
    expect(said).toBe("Una carpeta de Agentes se queda dentro de Agentes, donde la lee su agente.");
  });
});
