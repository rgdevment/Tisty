import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Folded, Papers } from "../core";
import { docChoices, folderChoices, type Hands, hereChoices, litFor } from "../docMenus";
import { AGENTS, isHidden, showAll } from "../hiding";
import { t } from "../locales";
import type { Choice } from "../ui/Menu";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    calls.push({ cmd, args });
    if (cmd === "doc_read") return Promise.resolve({ body: "# Hola" });
    if (cmd === "doc_export") return Promise.resolve({ files: 2, missed: 0, left: 0 });
    if (cmd === "doc_copy") return Promise.resolve({ id: "d9" });
    if (cmd === "agents_state") return agentsAnswer.now();
    if (cmd === "agents_turn" && agentsAnswer.refuses) return Promise.reject({ code: "x" });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve("C:/fuera"),
}));

const agentsAnswer = vi.hoisted(() => ({
  refuses: false,
  now: (): Promise<unknown> =>
    Promise.resolve({ folder: "a1", consented: true, allOn: true, assistants: [] }),
}));

const linking = vi.hoisted(() => ({ files: [] as string[], refuses: false }));

vi.mock("../copying", () => ({
  asPlain: () => Promise.resolve(),
  asLink: (file: string) => {
    linking.files.push(file);
    return linking.refuses ? Promise.reject({ code: "noClipboard" }) : Promise.resolve();
  },
}));

const folder = { id: "f1", name: "Casa", away: false, archived: false } as unknown as Folded;
const agentsRoot = {
  id: "a1",
  name: "AGENTS",
  icon: "robot",
  parent: null,
  away: false,
  archived: false,
} as unknown as Folded;
const agentsClaude = {
  id: "a2",
  name: "Claude",
  icon: null,
  parent: "a1",
  away: false,
  archived: false,
} as unknown as Folded;
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
    newDiagram: vi.fn(),
    bringIn: vi.fn(),
    makeFolder: vi.fn(),
    makeFolderIn: vi.fn(),
    rename: vi.fn(),
    parcels: { takeParcel: vi.fn(), packUp: vi.fn() },
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
  linking.files.length = 0;
  linking.refuses = false;
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
    pick(choices, "newDiagram").onPick?.();
    pick(hereChoices(h), "newDiagram").onPick?.();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.newDiagram.mock.calls).toEqual([["f1"], [undefined]]);
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.makeFolder).toHaveBeenCalled();
    expect(h.parcels.takeParcel).toHaveBeenCalled();
    expect(h.parcels.packUp).toHaveBeenCalledWith([], "tisty");
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
    pick(choices, "away").onPick?.();
    pick(choices, "drop").onPick?.();
    pick(choices, "newDiagram").onPick?.();
    await flush();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.newDiagram).toHaveBeenCalledWith("f1");
    expect(h.makeFolderIn).toHaveBeenCalledWith("f1");
    expect(h.rename).toHaveBeenCalledWith(folder);
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.dropFolder).toHaveBeenCalledWith(folder);
    expect(sent()).toContain("folder_away");
    expect(h.changed).toHaveBeenCalled();
  });
});

describe("the menu inside AGENTS", () => {
  it("brings another agent file from the AGENTS folders alone", () => {
    const h = hands();
    h.papers = { folders: [folder, agentsRoot, agentsClaude], docs: [] } as unknown as Papers;
    const asked = vi.fn();
    window.addEventListener("tisty:agents-more", asked);
    pick(folderChoices(agentsClaude, h), "agentsMore").onPick?.();
    pick(hereChoices(h, "a1"), "agentsMore").onPick?.();
    window.removeEventListener("tisty:agents-more", asked);
    expect(asked).toHaveBeenCalledTimes(2);
    expect(folderChoices(folder, h).some((one) => one.key === "agentsMore")).toBe(false);
    expect(hereChoices(h).some((one) => one.key === "agentsMore")).toBe(false);
    expect(pick(folderChoices({ ...agentsClaude, away: true }, h), "agentsMore").off).toBe(true);
  });

  it("turns AGENTS off from its own folder while it is on, and nowhere else", async () => {
    const h = hands();
    h.papers = { folders: [folder, agentsRoot, agentsClaude], docs: [] } as unknown as Papers;
    const has = (choices: Choice[]) => choices.some((one) => one.key === "agentsOff");
    expect(has(folderChoices(agentsRoot, h, false))).toBe(false);
    expect(has(folderChoices(agentsClaude, h, true))).toBe(false);
    expect(has(folderChoices(folder, h, true))).toBe(false);
    const heard = vi.fn();
    window.addEventListener("tisty:agents-changed", heard);
    const off = pick(folderChoices(agentsRoot, h, true), "agentsOff");
    expect(off.label).toBe("Turn AGENTS off");
    off.onPick?.();
    await flush();
    window.removeEventListener("tisty:agents-changed", heard);
    expect(calls).toEqual([{ cmd: "agents_turn", args: { assistant: null, on: false } }]);
    expect(h.changed).toHaveBeenCalled();
    expect(heard).toHaveBeenCalled();
  });

  it("asks whether AGENTS is on only for its own folder's menu", async () => {
    const folders = [folder, agentsRoot, agentsClaude];
    const then = vi.fn();
    litFor(folder, folders, then);
    expect(then).toHaveBeenLastCalledWith(false);
    expect(sent()).toEqual([]);
    litFor(agentsRoot, folders, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith(true);
    agentsAnswer.now = () => Promise.reject({ code: "x" });
    litFor(agentsRoot, folders, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith(true);
  });

  it("says so when turning AGENTS off fails", async () => {
    agentsAnswer.refuses = true;
    const h = hands();
    h.papers = { folders: [agentsRoot], docs: [] } as unknown as Papers;
    pick(folderChoices(agentsRoot, h, true), "agentsOff").onPick?.();
    await flush();
    agentsAnswer.refuses = false;
    expect(h.fail).toHaveBeenCalled();
    expect(h.changed).not.toHaveBeenCalled();
  });

  it("offers no Hide on AGENTS while it is on, only Turn AGENTS off", () => {
    const h = hands();
    h.papers = { folders: [folder, agentsRoot], docs: [] } as unknown as Papers;
    const keys = folderChoices(agentsRoot, h, true).map((one) => one.key);
    expect(keys).toContain("agentsOff");
    expect(keys).not.toContain("hide");
    expect(folderChoices(folder, h, true).map((one) => one.key)).toContain("hide");
  });

  it("hides AGENTS under its own key, so the row stays hidden once its folder is made again", () => {
    showAll();
    const h = hands();
    h.papers = { folders: [folder, agentsRoot], docs: [] } as unknown as Papers;
    pick(folderChoices(agentsRoot, h), "hide").onPick?.();
    expect(isHidden(AGENTS)).toBe(true);
    expect(isHidden("a1")).toBe(false);
    pick(folderChoices(agentsRoot, h), "hide").onPick?.();
    expect(isHidden(AGENTS)).toBe(false);
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

  it("copies the link of that document and says so", async () => {
    const h = hands();

    pick(docChoices(other, h), "asLink").onPick?.();
    await flush();

    expect(linking.files).toEqual(["dev_a-0002"]);
    expect(h.noted).toHaveBeenCalledWith(t("linkCopied"));
    expect(h.fail).not.toHaveBeenCalled();
  });

  it("reports a link that could not be copied instead of saying it was", async () => {
    const h = hands();
    linking.refuses = true;

    pick(docChoices(doc, h), "asLink").onPick?.();
    await flush();

    expect(h.fail).toHaveBeenCalledWith({ code: "noClipboard" });
    expect(h.noted).not.toHaveBeenCalled();
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

  it("offers the idea board only its copy, its shelving, hiding it and its deletion", () => {
    const board = docChoices(doc, hands(), true);

    expect(board.map((one) => one.key)).toEqual(["copy", "away", "hide", "drop"]);
    expect(board[0]?.apart).toBe(false);
  });
});
