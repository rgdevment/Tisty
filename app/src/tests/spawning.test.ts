import { describe, expect, it, vi, beforeEach } from "vitest";

const store = vi.hoisted(() => ({
  folders: [] as (string | undefined)[],
  wrote: [] as { id: string; body: string }[],
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
}));

import { spawned } from "../making";

beforeEach(() => {
  store.folders = [];
  store.wrote = [];
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

  it("gives back the reference already written, ready to insert", async () => {
    const born = await spawned("Minuta", "01H");

    expect(born.said).toBe("[Minuta](tisty:doc/mac0-0001)");
    expect(born.id).toBe("mac0-0001");
  });

  it("makes nothing at all when the name is only spaces", async () => {
    await expect(spawned("   ", "01H")).rejects.toThrow();

    expect(store.folders).toEqual([]);
    expect(store.wrote).toEqual([]);
  });
});
