import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Folded, Papers } from "../core";
import { docChoices, folderChoices, type Hands, hereChoices, stageFor } from "../docMenus";
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
    Promise.resolve({ folder: "a1", held: 0, consented: true, allOn: true, assistants: [] }),
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

  const withRoot = (h: ReturnType<typeof hands>) => {
    h.papers = { folders: [folder, agentsRoot, agentsClaude], docs: [] } as unknown as Papers;
    return h;
  };
  const keys = (choices: Choice[]) => choices.map((one) => one.key);

  it("offers only Sync and Hide on a brought Agents", () => {
    const h = withRoot(hands());
    const choices = folderChoices(agentsRoot, h, "brought");
    expect(keys(choices)).toEqual(["agentsSync", "hide"]);
    const sync = pick(choices, "agentsSync");
    expect(sync.label).toBe("Sync");
    const heard = vi.fn();
    window.addEventListener("tisty:agents-sync", heard);
    sync.onPick?.();
    window.removeEventListener("tisty:agents-sync", heard);
    expect(heard).toHaveBeenCalledTimes(1);
    expect(sent()).toEqual([]);
  });

  it("offers only Stop syncing on an Agents that syncs, and on one whose stage is unknown", async () => {
    const h = withRoot(hands());
    const choices = folderChoices(agentsRoot, h, "syncs");
    expect(keys(choices)).toEqual(["agentsUnsync"]);
    expect(keys(folderChoices(agentsRoot, h))).toEqual(["agentsUnsync"]);
    const heard = vi.fn();
    window.addEventListener("tisty:agents-changed", heard);
    const stop = pick(choices, "agentsUnsync");
    expect(stop.label).toBe("Stop syncing");
    stop.onPick?.();
    await flush();
    window.removeEventListener("tisty:agents-changed", heard);
    expect(calls).toEqual([{ cmd: "agents_turn", args: { assistant: null, on: false } }]);
    expect(h.changed).toHaveBeenCalled();
    expect(heard).toHaveBeenCalled();
  });

  it("offers only Hide on an Agents with nothing brought", () => {
    const h = withRoot(hands());
    expect(keys(folderChoices(agentsRoot, h, "unbrought"))).toEqual(["hide"]);
  });

  it("offers no folder action on Agents itself, whatever its stage, and every one below it", () => {
    const h = withRoot(hands());
    for (const stage of ["unbrought", "brought", "syncs", null] as const) {
      const choices = keys(folderChoices(agentsRoot, h, stage));
      for (const one of ["newDoc", "newFolder", "rename", "move", "agentsMore", "away", "drop"])
        expect(choices).not.toContain(one);
    }
    const below = keys(folderChoices(agentsClaude, h));
    expect(below).toEqual(expect.arrayContaining(["rename", "agentsMore", "drop"]));
    expect(pick(folderChoices(agentsClaude, h), "rename").off).toBe(false);
  });

  it("offers no sync choice on any other folder, whatever stage it is given", () => {
    const h = withRoot(hands());
    for (const stage of ["brought", "syncs"] as const) {
      for (const one of [folder, agentsClaude]) {
        const choices = keys(folderChoices(one, h, stage));
        expect(choices).not.toContain("agentsSync");
        expect(choices).not.toContain("agentsUnsync");
        expect(choices).toContain("hide");
      }
    }
  });

  it("asks for the stage only on Agents' own menu, and counts unknown as syncing", async () => {
    const at = { folders: [folder, agentsRoot, agentsClaude], docs: [] } as unknown as Papers;
    const then = vi.fn();
    stageFor(folder, at, then);
    expect(then).toHaveBeenLastCalledWith(null);
    stageFor(agentsClaude, at, then);
    expect(then).toHaveBeenLastCalledWith(null);
    expect(sent()).toEqual([]);
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("syncs");
    agentsAnswer.now = () =>
      Promise.resolve({ folder: "a1", held: 0, consented: false, allOn: true, assistants: [] });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("unbrought");
    agentsAnswer.now = () =>
      Promise.resolve({ folder: "a1", held: 1, consented: false, allOn: true, assistants: [] });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("brought");
    agentsAnswer.now = () => Promise.reject({ code: "x" });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("syncs");
    agentsAnswer.now = () =>
      Promise.resolve({ folder: "a1", held: 0, consented: true, allOn: true, assistants: [] });
  });

  it("says so when stopping the sync fails", async () => {
    agentsAnswer.refuses = true;
    const h = hands();
    h.papers = { folders: [agentsRoot], docs: [] } as unknown as Papers;
    pick(folderChoices(agentsRoot, h, "syncs"), "agentsUnsync").onPick?.();
    await flush();
    agentsAnswer.refuses = false;
    expect(h.fail).toHaveBeenCalled();
    expect(h.changed).not.toHaveBeenCalled();
  });

  it("hides AGENTS under its own key, so the row stays hidden once its folder is made again", () => {
    showAll();
    const h = hands();
    h.papers = { folders: [folder, agentsRoot], docs: [] } as unknown as Papers;
    pick(folderChoices(agentsRoot, h, "brought"), "hide").onPick?.();
    expect(isHidden(AGENTS)).toBe(true);
    expect(isHidden("a1")).toBe(false);
    pick(folderChoices(agentsRoot, h, "brought"), "hide").onPick?.();
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
