import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Folded, Papers } from "../core";
import { docChoices, folderChoices, type Hands, hereChoices } from "../docMenus";
import type { Choice } from "../ui/Menu";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    calls.push({ cmd, args });
    if (cmd === "doc_read") return Promise.resolve({ body: "# Hola" });
    if (cmd === "doc_export") return Promise.resolve({ files: 2, missed: 0, left: 0 });
    if (cmd === "doc_copy") return Promise.resolve({ id: "d9" });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve("C:/fuera"),
}));

vi.mock("../copying", () => ({ asPlain: () => Promise.resolve() }));

const folder = { id: "f1", name: "Casa", away: false, archived: false } as unknown as Folded;
const doc = {
  id: "d1",
  file: "dev_a-0001",
  title: "Minuta",
  folder: "f1",
  away: false,
  archived: false,
  locked: false,
} as unknown as Filed;
const other = { ...doc, id: "d2", file: "dev_a-0002", title: "Otra" } as unknown as Filed;
const papers = { folders: [folder], docs: [doc, other] } as unknown as Papers;

const hands = () => {
  const h = {
    papers,
    showing: doc.file,
    newDoc: vi.fn(),
    bringIn: vi.fn(),
    makeFolder: vi.fn(),
    makeFolderIn: vi.fn(),
    rename: vi.fn(),
    parcels: { takeParcel: vi.fn(), packUp: vi.fn(), takeOutAll: vi.fn() },
    changed: vi.fn(),
    fail: vi.fn(),
    failWith: vi.fn(),
    noted: vi.fn(),
    open: vi.fn(),
    dropFolder: vi.fn(),
    dropDoc: vi.fn(),
    bringBack: vi.fn(),
    hangIt: vi.fn(),
  };
  return h as typeof h & Hands;
};

const pick = (choices: Choice[], key: string) => {
  const one = choices.find((choice) => choice.key === key);
  if (!one) throw new Error(`no ${key}`);
  return one;
};
const flush = () => new Promise((ready) => setTimeout(ready, 0));
const sent = () => calls.map((one) => one.cmd);

beforeEach(() => {
  calls.length = 0;
});

describe("the menu of the documents themselves", () => {
  it("makes a document or brings one in where the person stands", () => {
    const h = hands();
    const choices = hereChoices(h, "f1");

    pick(choices, "newDoc").onPick?.();
    pick(choices, "import").onPick?.();
    pick(choices, "newFolder").onPick?.();
    pick(choices, "unpack").onPick?.();
    pick(choices, "packAll").onPick?.();
    pick(choices, "takeOutAll").onPick?.();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.makeFolder).toHaveBeenCalled();
    expect(h.parcels.takeParcel).toHaveBeenCalled();
    expect(h.parcels.packUp).toHaveBeenCalledWith([], "tisty");
    expect(h.parcels.takeOutAll).toHaveBeenCalled();
  });
});

describe("the menu of a folder", () => {
  it("does each thing to that folder", async () => {
    const h = hands();
    const choices = folderChoices(folder, h);

    pick(choices, "newDoc").onPick?.();
    pick(choices, "newFolder").onPick?.();
    pick(choices, "rename").onPick?.();
    pick(choices, "import").onPick?.();
    pick(choices, "unpack").onPick?.();
    pick(choices, "packAll").onPick?.();
    pick(choices, "takeOutAll").onPick?.();
    pick(choices, "away").onPick?.();
    pick(choices, "drop").onPick?.();
    await flush();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.makeFolderIn).toHaveBeenCalledWith("f1");
    expect(h.rename).toHaveBeenCalledWith(folder);
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.dropFolder).toHaveBeenCalledWith(folder);
    expect(sent()).toContain("folder_away");
    expect(h.changed).toHaveBeenCalled();
  });
});

describe("the menu of a document", () => {
  it("does each thing to that document", async () => {
    const h = hands();
    const choices = docChoices(doc, h);

    pick(choices, "newPage").onPick?.();
    pick(choices, "pageOf").into?.choices[0]?.onPick?.();
    pick(choices, "asPlain").onPick?.();
    pick(choices, "takeOut").onPick?.();
    pick(choices, "packIt").onPick?.();
    pick(choices, "copy").onPick?.();
    pick(choices, "lock").onPick?.();
    pick(choices, "away").onPick?.();
    pick(choices, "drop").onPick?.();
    await flush();
    await flush();

    expect(h.newDoc).toHaveBeenCalledWith(undefined, "d1");
    expect(h.hangIt).toHaveBeenCalledWith("d1", "d2");
    expect(h.parcels.packUp).toHaveBeenCalledWith(["dev_a-0001"], "Minuta");
    expect(h.open).toHaveBeenCalledWith("d9");
    expect(h.dropDoc).toHaveBeenCalledWith(doc);
    expect(h.noted).toHaveBeenCalledTimes(2);
    expect(sent()).toEqual(
      expect.arrayContaining(["doc_export", "doc_copy", "doc_lock", "doc_away"]),
    );
  });

  it("brings an archived document back instead of putting it away", () => {
    const h = hands();

    pick(docChoices({ ...doc, archived: true } as Filed, h), "away").onPick?.();

    expect(h.bringBack).toHaveBeenCalled();
    expect(sent()).not.toContain("doc_away");
  });

  it("offers the PDF only for the document on screen", () => {
    const h = hands();

    expect(pick(docChoices(other, h), "seePdf").off).toBe(true);
    expect(pick(docChoices(doc, h), "seePdf").off).toBe(false);
  });
});
