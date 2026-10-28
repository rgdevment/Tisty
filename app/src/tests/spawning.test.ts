import { beforeEach, describe, expect, it, vi } from "vitest";

const store = vi.hoisted(() => ({
  folders: [] as (string | undefined)[],
  wrote: [] as { id: string; body: string }[],
  added: [] as string[],
  shelves: [] as unknown[],
  listed: 0,
}));

vi.mock("../core", () => ({
  docNew: (folder?: string) => {
    store.folders.push(folder);
    return Promise.resolve({ id: `mac0-000${store.folders.length}`, title: "" });
  },
  docWrite: (id: string, body: string) => {
    store.wrote.push({ id, body });
    return Promise.resolve({ id, title: "" });
  },
  docs: () => {
    store.listed += 1;
    return Promise.resolve({ folders: store.shelves, docs: [] });
  },
  folderAdd: (name: string) => {
    store.added.push(name);
    return Promise.resolve("01NEW");
  },
}));

import { startOf } from "tisty-diagrams/read";
import type { Folded } from "../core";
import { t } from "../locales";
import { drafted, placeOf, spawned } from "../making";

beforeEach(() => {
  store.folders = [];
  store.wrote = [];
  store.added = [];
  store.shelves = [];
  store.listed = 0;
});

describe("making a document that another one will point at", () => {
  it("puts it in the folder it was asked to, so it lands beside its parent", async () => {
    await spawned("Anexo del contrato", "01H");

    expect(store.folders).toEqual(["01H"]);
  });

  it("leaves it unfiled when there is no folder to inherit", async () => {
    await spawned("Suelto");

    expect(store.folders).toEqual([undefined]);
  });

  it("writes the name as the title, so it is not born untitled", async () => {
    await spawned("Minuta del lunes", "01H");

    expect(store.wrote[0].body).toBe("# Minuta del lunes\n\n");
  });

  it("gives back a card already written, ready to insert", async () => {
    const born = await spawned("Minuta", "01H");

    expect(born.said).toBe("![Minuta](tisty:doc/mac0-0001)");
    expect(born.id).toBe("mac0-0001");
  });

  it("makes nothing at all when the name is only spaces", async () => {
    await expect(spawned("   ", "01H")).rejects.toThrow();

    expect(store.folders).toEqual([]);
    expect(store.wrote).toEqual([]);
  });
});

const shelf = (id: string, name: string, more: Partial<Folded> = {}) =>
  ({ id, name, parent: null, away: false, archived: false, ...more }) as unknown as Folded;

describe("making a document for a new diagram", () => {
  it("writes its name as the title and the example of its kind below, and says where the block starts", async () => {
    const made = await drafted("Pagos ñandú", "flowchart", "01H");

    expect(store.folders).toEqual(["01H"]);
    expect(store.wrote[0].body).toBe(
      `# Pagos ñandú\n\n\`\`\`mermaid\n${startOf("flowchart", "en")}\`\`\`\n`,
    );
    expect(made).toMatchObject({ id: "mac0-0001", title: "Pagos ñandú", folder: "01H", at: 17 });
    expect(store.listed).toBe(0);
    expect(made.source).toBe(startOf("flowchart", "en"));
  });

  it("puts it in the diagrams' folder as the disk lists it now, made only when there is none at the top", async () => {
    const home = shelf("01D", t("diagramsFolder"));
    const below = shelf("01B", t("diagramsFolder"), { parent: "01X" } as Partial<Folded>);
    const away = shelf("01A", t("diagramsFolder"), { away: true });

    store.shelves = [below, away, home];
    const found = await drafted("Uno", "mindmap");
    store.shelves = [below, away];
    const missing = await drafted("Dos", "mindmap");

    expect(store.folders).toEqual(["01D", "01NEW"]);
    expect(store.added).toEqual([t("diagramsFolder")]);
    expect([found.folder, missing.folder]).toEqual(["01D", "01NEW"]);
    expect(store.listed).toBe(2);
    expect(placeOf([home], "01D")).toBe(t("diagramsFolder"));
    expect(placeOf([], "01NEW")).toBe(t("diagramsFolder"));
    expect(placeOf([shelf("01C", "Casa")], "01C")).toBe("Casa");
  });

  it("makes nothing when the name is only spaces", async () => {
    await expect(drafted("  ", "mindmap")).rejects.toThrow();

    expect(store.folders).toEqual([]);
    expect(store.added).toEqual([]);
  });
});
