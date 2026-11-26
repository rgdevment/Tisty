import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsState } from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
import { docChoices, folderChoices, type Hands, hereChoices, stageFor } from "../docMenus";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { t } from "../locales";
import { asksSure } from "../sure";
import type { Choice } from "../ui/Menu";

const calls: { cmd: string; args: Record<string, unknown> }[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    calls.push({ cmd, args });
    if (cmd === "doc_read") return Promise.resolve({ body: "# Hola" });
    if (cmd === "doc_export") return Promise.resolve({ files: 2, missed: 0, left: 0 });
    if (cmd === "doc_copy") return Promise.resolve({ id: "d9" });
    if (cmd === "agents_state") return agentsAnswer.now();
    if (cmd === "agents_moving") return Promise.resolve(movingAnswer.now);
    if (cmd === "agents_turn" && agentsAnswer.refuses) return Promise.reject({ code: "x" });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve("C:/fuera"),
}));

const movingAnswer = vi.hoisted(() => ({ now: null as unknown }));

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
const agentsGlobal = { ...agentsClaude, id: "a3", name: "Global" } as unknown as Folded;
const ownFolder = { ...agentsClaude, id: "a4", name: "Mine", parent: "a2" } as unknown as Folded;
const lit = (on = true): AgentsState => ({
  folder: "a1",
  held: 1,
  consented: true,
  allOn: true,
  assistants: [{ id: "claude", name: "Claude", on }],
});
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
const into = (choices: Choice[], key: string, inner: string) =>
  pick(pick(choices, key).into?.choices ?? [], inner);
const flush = () => new Promise((ready) => setTimeout(ready, 0));
const sent = () => calls.map((one) => one.cmd);

beforeEach(() => {
  calls.length = 0;
  agentsAnswer.refuses = false;
  movingAnswer.now = null;
  asksSure(null);
  showAll();
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
    pick(hereChoices(h), "newFolder").onPick?.();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.newDiagram.mock.calls).toEqual([["f1"], [undefined]]);
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.makeFolderIn.mock.calls).toEqual([["f1"], [undefined]]);
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
    into(choices, "more", "import").onPick?.();
    pick(choices, "packFolder").onPick?.();
    pick(choices, "away").onPick?.();
    pick(choices, "drop").onPick?.();
    into(choices, "more", "newDiagram").onPick?.();
    await flush();

    expect(h.newDoc).toHaveBeenCalledWith("f1");
    expect(h.newDiagram).toHaveBeenCalledWith("f1");
    expect(h.makeFolderIn).toHaveBeenCalledWith("f1");
    expect(h.rename).toHaveBeenCalledWith(folder);
    expect(h.bringIn).toHaveBeenCalledWith("f1");
    expect(h.parcels.packUp).toHaveBeenCalledWith(["dev_a-0001", "dev_a-0002"], "Casa");
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

  it("offers Sync, Look again and Hide on a brought Agents", () => {
    const h = withRoot(hands());
    const choices = folderChoices(agentsRoot, h, "brought");
    expect(keys(choices)).toEqual(["agentsSync", "agentsRescan", "hide"]);
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
    expect(keys(choices)).toEqual(["agentsUnsync", "agentsRescan", "hide"]);
    expect(keys(folderChoices(agentsRoot, h))).toEqual(["agentsUnsync", "agentsRescan", "hide"]);
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

  it("looks for agent files again from the Agents menu", () => {
    const h = withRoot(hands());
    const heard = vi.fn();
    window.addEventListener("tisty:agents-rescan", heard);
    pick(folderChoices(agentsRoot, h, "syncs"), "agentsRescan").onPick?.();
    window.removeEventListener("tisty:agents-rescan", heard);
    expect(heard).toHaveBeenCalledTimes(1);
    expect(pick(folderChoices(agentsRoot, h, "syncs"), "agentsRescan").label).toBe("Look again");
  });

  it("offers Hide on Agents and every folder inside it, whatever the stage", () => {
    const h = withRoot(hands());
    for (const stage of ["unbrought", "brought", "syncs", null] as const) {
      expect(keys(folderChoices(agentsRoot, h, stage))).toContain("hide");
      expect(keys(folderChoices(agentsClaude, h, stage))).toContain("hide");
    }
    expect(keys(folderChoices(folder, h))).toContain("hide");
  });

  it("offers inside Agents only a new document, another file, pausing, hiding and deleting", () => {
    const h = withRoot(hands());
    h.papers = {
      folders: [folder, agentsRoot, agentsClaude, agentsGlobal, ownFolder],
      docs: [],
    } as unknown as Papers;
    const turning = ["newDoc", "agentsMore", "agentsTurn", "hide", "drop"];
    const plain = ["newDoc", "agentsMore", "hide", "drop"];
    expect(keys(folderChoices(agentsClaude, h, "syncs", lit()))).toEqual(turning);
    expect(keys(folderChoices(ownFolder, h, "syncs", lit()))).toEqual(turning);
    expect(keys(folderChoices(agentsGlobal, h, "syncs", lit()))).toEqual(plain);
    expect(keys(folderChoices(agentsClaude, h, "brought", lit()))).toEqual(plain);
    expect(keys(folderChoices(agentsClaude, h, "syncs", null))).toEqual(plain);
    expect(keys(folderChoices(agentsClaude, h, null))).toEqual(plain);
    const shelved = { ...agentsClaude, away: true, archived: true } as unknown as Folded;
    const back = folderChoices(shelved, h, "brought");
    expect(keys(back)).toEqual(["newDoc", "agentsMore", "hide", "away", "drop"]);
    expect(pick(back, "away").off).toBe(false);
    expect(pick(back, "newDoc").off).toBe(true);
  });

  it("pauses and resumes the assistant of the folder as its page does", async () => {
    const h = withRoot(hands());
    const pause = pick(folderChoices(agentsClaude, h, "syncs", lit()), "agentsTurn");
    expect(pause.label).toBe("Pause sync with Claude");
    pause.onPick?.();
    await flush();
    const resume = pick(folderChoices(agentsClaude, h, "syncs", lit(false)), "agentsTurn");
    expect(resume.label).toBe("Resume sync with Claude");
    resume.onPick?.();
    await flush();
    expect(calls).toEqual([
      { cmd: "agents_turn", args: { assistant: "claude", on: false } },
      { cmd: "agents_turn", args: { assistant: "claude", on: true } },
    ]);
    expect(h.changed).toHaveBeenCalledTimes(2);
  });

  it("makes the new document and deletes the folder from inside Agents", () => {
    const h = withRoot(hands());
    const choices = folderChoices(agentsClaude, h, "syncs", lit());
    pick(choices, "newDoc").onPick?.();
    pick(choices, "drop").onPick?.();
    expect(h.newDoc).toHaveBeenCalledWith("a2");
    expect(h.dropFolder).toHaveBeenCalledWith(agentsClaude);
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
    for (const one of ["newDiagram", "newFolder", "rename", "move", "import", "unpack", "packAll"])
      expect(below).not.toContain(one);
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

  it("asks for the stage only inside Agents, and counts unknown as syncing", async () => {
    const at = { folders: [folder, agentsRoot, agentsClaude], docs: [] } as unknown as Papers;
    const then = vi.fn();
    stageFor(folder, at, then);
    expect(then).toHaveBeenLastCalledWith(null, null);
    expect(sent()).toEqual([]);
    stageFor(agentsClaude, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("syncs", expect.objectContaining({ allOn: true }));
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("syncs", expect.objectContaining({ allOn: true }));
    agentsAnswer.now = () =>
      Promise.resolve({ folder: "a1", held: 0, consented: false, allOn: true, assistants: [] });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("unbrought", expect.anything());
    agentsAnswer.now = () =>
      Promise.resolve({ folder: "a1", held: 1, consented: false, allOn: true, assistants: [] });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("brought", expect.anything());
    agentsAnswer.now = () => Promise.reject({ code: "x" });
    stageFor(agentsRoot, at, then);
    await flush();
    expect(then).toHaveBeenLastCalledWith("syncs", null);
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

  const answering = (yes: boolean) => {
    const asked: unknown[] = [];
    asksSure((said, buttons) => {
      asked.push({ said, buttons });
      return Promise.resolve(yes);
    });
    return asked;
  };

  it("asks to stop syncing before hiding a syncing Agents, and stops first", async () => {
    const h = withRoot(hands());
    const asked = answering(true);
    pick(folderChoices(agentsRoot, h, "syncs"), "hide").onPick?.();
    await flush();
    expect(asked).toEqual([
      {
        said: "To hide Agents, first stop syncing. Your agent files are not touched.",
        buttons: { yes: "Stop syncing and hide", no: "Cancel" },
      },
    ]);
    expect(calls).toEqual([{ cmd: "agents_turn", args: { assistant: null, on: false } }]);
    expect(isHidden(AGENTS)).toBe(true);
    expect(h.changed).toHaveBeenCalled();
  });

  it("leaves Agents shown and syncing when the person cancels, the unknown stage asking too", async () => {
    const h = withRoot(hands());
    const asked = answering(false);
    pick(folderChoices(agentsRoot, h, null), "hide").onPick?.();
    pick(folderChoices(agentsClaude, h, "syncs", lit()), "hide").onPick?.();
    await flush();
    expect(asked).toHaveLength(2);
    expect(calls).toEqual([]);
    expect(isHidden(AGENTS)).toBe(false);
    expect(isHidden("a2")).toBe(false);
  });

  it("hides a folder inside a syncing Agents only after stopping the sync", async () => {
    const h = withRoot(hands());
    answering(true);
    pick(folderChoices(agentsClaude, h, "syncs", lit()), "hide").onPick?.();
    await flush();
    expect(calls).toEqual([{ cmd: "agents_turn", args: { assistant: null, on: false } }]);
    expect(isHidden("a2")).toBe(true);
  });

  it("never hides when stopping the sync fails", async () => {
    agentsAnswer.refuses = true;
    const h = withRoot(hands());
    answering(true);
    pick(folderChoices(agentsClaude, h, "syncs", lit()), "hide").onPick?.();
    await flush();
    expect(isHidden("a2")).toBe(false);
    expect(h.fail).toHaveBeenCalled();
  });

  it("hides without asking once Agents does not sync, and shows again without asking", () => {
    const h = withRoot(hands());
    const asked = answering(true);
    pick(folderChoices(agentsClaude, h, "brought"), "hide").onPick?.();
    expect(isHidden("a2")).toBe(true);
    hide(AGENTS);
    pick(folderChoices(agentsRoot, h, "syncs"), "hide").onPick?.();
    expect(isHidden(AGENTS)).toBe(false);
    expect(asked).toEqual([]);
    expect(calls).toEqual([]);
  });
});

describe("the menu of a document", () => {
  it("offers the width of the text, in its own submenu", () => {
    const choice = pick(docChoices(doc, hands()), "measure");

    expect(choice.into?.choices).toHaveLength(4);
  });

  it("does each thing to that document", async () => {
    const h = hands();
    const choices = docChoices(doc, h);

    pick(choices, "newPage").onPick?.();
    into(choices, "move", "pageOf").into?.choices[0]?.onPick?.();
    into(choices, "copyAs", "asPlain").onPick?.();
    into(choices, "exportAs", "takeOut").onPick?.();
    into(choices, "exportAs", "packIt").onPick?.();
    into(choices, "copyAs", "copy").onPick?.();
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

    into(docChoices(other, h), "copyAs", "asLink").onPick?.();
    await flush();

    expect(linking.files).toEqual(["dev_a-0002"]);
    expect(h.noted).toHaveBeenCalledWith(t("linkCopied"));
    expect(h.fail).not.toHaveBeenCalled();
  });

  it("reports a link that could not be copied instead of saying it was", async () => {
    const h = hands();
    linking.refuses = true;

    into(docChoices(doc, h), "copyAs", "asLink").onPick?.();
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

    const away = into(docChoices(other, h), "exportAs", "seePdf");
    const here = into(docChoices(doc, h), "exportAs", "seePdf");

    expect(away.hint).toBe(t("openToExport"));
    away.onPick?.();
    expect(h.failWith).toHaveBeenCalledWith(t("openToExport"));
    expect(here.hint).toBeUndefined();
  });

  it("offers the idea board only its shelving, hiding it and its deletion", () => {
    const board = docChoices(doc, hands(), true);

    expect(board.map((one) => one.key)).toEqual(["away", "hide", "drop"]);
    expect(board[0]?.apart).toBe(false);
  });
});

describe("the menus of the folders and documents of the computers in Agents", () => {
  const mineFolder = {
    ...agentsClaude,
    id: "m1",
    name: "desk",
    parent: "a1",
    agents: { computer: "Desk", mine: true, room: false },
  } as unknown as Folded;
  const mineClaude = {
    ...agentsClaude,
    id: "m2",
    parent: "m1",
    agents: { mine: true, room: true },
  } as unknown as Folded;
  const theirFolder = {
    ...agentsClaude,
    id: "o1",
    name: "laptop",
    parent: "a1",
    agents: { computer: "Laptop", mine: false, room: false },
  } as unknown as Folded;
  const theirClaude = {
    ...agentsClaude,
    id: "o2",
    parent: "o1",
    agents: { mine: false, room: true },
  } as unknown as Folded;
  const theirRule = { ...doc, id: "d3", file: "dev_a-0003", folder: "o2" } as unknown as Filed;
  const mineRule = { ...doc, id: "d4", file: "dev_a-0004", folder: "m2" } as unknown as Filed;

  const inComputers = () => {
    const h = hands();
    h.papers = {
      folders: [folder, agentsRoot, mineFolder, mineClaude, theirFolder, theirClaude],
      docs: [theirRule, mineRule],
    } as unknown as Papers;
    return h;
  };
  const keys = (choices: Choice[]) => choices.map((one) => one.key);

  it("offers to pause an assistant inside this computer's folder, and not the computer's folder", () => {
    const h = inComputers();
    expect(keys(folderChoices(mineClaude, h, "syncs", lit()))).toContain("agentsTurn");
    expect(keys(folderChoices(mineFolder, h, "syncs", lit()))).not.toContain("agentsTurn");
  });

  it("pauses the assistant whose folder it is from inside the computer's folder", async () => {
    const h = inComputers();
    pick(folderChoices(mineClaude, h, "syncs", lit()), "agentsTurn").onPick?.();
    await flush();
    expect(calls).toEqual([{ cmd: "agents_turn", args: { assistant: "claude", on: false } }]);
  });

  it("offers only Hide on every folder of another computer, whatever the stage", () => {
    const h = inComputers();
    for (const stage of ["syncs", "brought", null] as const) {
      expect(keys(folderChoices(theirFolder, h, stage, lit()))).toEqual(["hide"]);
      expect(keys(folderChoices(theirClaude, h, stage, lit()))).toEqual(["hide"]);
    }
  });

  it("hides the folder of another computer and nothing else", () => {
    const h = inComputers();
    pick(folderChoices(theirClaude, h, "brought", lit()), "hide").onPick?.();
    expect(isHidden("o2")).toBe(true);
    expect(calls).toEqual([]);
  });

  it("offers on a document of another computer only to move, copy as text or link, take out and pack", () => {
    const h = inComputers();
    const choices = docChoices(theirRule, h);
    const all = keys(choices);
    expect(all.sort()).toEqual(["copyAs", "exportAs", "move"]);
    const copies = pick(choices, "copyAs").into?.choices ?? [];
    expect(keys(copies)).toEqual(["copy", "asPlain", "asLink"]);
    expect(pick(copies, "copy").hint).toBe(t("notHereSaid"));
    expect(keys(pick(choices, "exportAs").into?.choices ?? [])).toEqual(
      expect.arrayContaining(["takeOut", "packIt"]),
    );
    expect(all).not.toContain("drop");
  });

  it("leaves a document of this computer with its whole menu", () => {
    const h = inComputers();
    const mineAll = keys(docChoices(mineRule, h));
    expect(mineAll).toEqual(expect.arrayContaining(["drop", "copyAs", "lock", "away", "newPage"]));
  });

  it("offers no text width on an agent's file, which shows as its card", () => {
    const h = inComputers();
    expect(keys(docChoices(mineRule, h))).not.toContain("measure");
    expect(keys(docChoices(theirRule, h))).not.toContain("measure");
  });
});

describe("moving inside Agents asks first", () => {
  const claude = { ...agentsClaude, id: "a2" } as unknown as Folded;
  const preview = {
    from: { is: "everyone" },
    to: { is: "mine" },
    docs: [{ title: "Minuta", file: "AGENTS.md", here: "keeps" }],
    older: [],
  };
  const inAgents = () => {
    const h = hands();
    h.papers = { folders: [folder, agentsRoot, claude], docs: [doc] } as unknown as Papers;
    return h;
  };
  const files = () => sent().filter((cmd) => cmd === "doc_file" || cmd === "folder_file");
  const land = (choices: Choice[]) => pick(pick(choices, "move").into?.choices ?? [], "a2");

  it("does not move a document when the answer is no", async () => {
    movingAnswer.now = preview;
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(false);
    });
    const h = inAgents();
    land(docChoices(doc, h)).onPick?.();
    await flush();
    expect(asked).toHaveLength(1);
    expect(sent().filter((cmd) => cmd === "agents_moving")).toHaveLength(1);
    expect(files()).toEqual([]);
    expect(h.changed).not.toHaveBeenCalled();
  });

  it("moves a document when the answer is yes", async () => {
    movingAnswer.now = preview;
    asksSure(() => Promise.resolve(true));
    const h = inAgents();
    land(docChoices(doc, h)).onPick?.();
    await flush();
    expect(files()).toEqual(["doc_file"]);
    expect(h.changed).toHaveBeenCalledTimes(1);
  });

  it("never offers Agents as a place for a plain folder, which it does not take", () => {
    const h = inAgents();
    const offered = (pick(folderChoices(folder, h), "move").into?.choices ?? []).map(
      (one) => one.key,
    );
    expect(offered).not.toContain("a2");
  });

  it("moves without asking when the core has nothing to say", async () => {
    const asked = vi.fn(() => Promise.resolve(false));
    asksSure(asked);
    const h = inAgents();
    land(docChoices(doc, h)).onPick?.();
    await flush();
    expect(asked).not.toHaveBeenCalled();
    expect(files()).toEqual(["doc_file"]);
  });
});

describe("the one shape of the menus", () => {
  const keysOf = (choices: Choice[]) => choices.map((one) => one.key);

  it("keeps the same choices in the same order for every kind of document", () => {
    const h = hands();
    const page = { ...other, pageOf: "d1" } as Filed;
    const shapes = [
      doc,
      { ...doc, locked: true },
      { ...doc, away: true, archived: true },
      { ...doc, away: true },
      page,
    ].map((one) => keysOf(docChoices(one as Filed, h)));

    for (const shape of shapes) expect(shape).toEqual(shapes[0]);
  });

  it("keeps a choice that does not apply in its place, with the reason, and refuses on pick", () => {
    const h = hands();
    const locked = docChoices({ ...doc, locked: true } as Filed, h);

    for (const key of ["newPage", "drop"]) {
      const one = pick(locked, key);
      expect(one.hint).toBe(t("lockedSaid"));
      expect(one.into).toBeUndefined();
      one.onPick?.();
    }
    expect(h.failWith).toHaveBeenCalledTimes(2);
    expect(h.failWith).toHaveBeenCalledWith(t("lockedSaid"));
    expect(h.dropDoc).not.toHaveBeenCalled();
    expect(h.newDoc).not.toHaveBeenCalled();
  });

  it("keeps the choices of a folder in the archive in place and refuses what it cannot do", () => {
    const h = hands();
    const shelved = { ...folder, away: true, archived: true } as unknown as Folded;
    const live = folderChoices(folder, h);
    const kept = folderChoices(shelved, h);

    expect(keysOf(kept)).toEqual(keysOf(live));
    for (const key of ["newDoc", "newFolder", "more", "rename", "move"]) {
      expect(pick(kept, key).hint).toBe(t("inArchiveSaid"));
      pick(kept, key).onPick?.();
    }
    expect(h.failWith).toHaveBeenCalledTimes(5);
    expect(h.newDoc).not.toHaveBeenCalled();
    expect(h.rename).not.toHaveBeenCalled();
  });

  it("keeps a folder with nothing in it from offering to export it", () => {
    const h = hands();
    h.papers = { folders: [folder], docs: [] } as unknown as Papers;
    const one = pick(folderChoices(folder, h), "packFolder");

    expect(one.hint).toBe(t("folderEmptySaid"));
    one.onPick?.();
    expect(h.parcels.packUp).not.toHaveBeenCalled();
  });

  it("keeps a diary book to copy, export and archive, and Ideas to archive, hide and delete", () => {
    const h = hands();
    const book = { ...doc, id: "b1", file: "dev_a-0003", diary: true } as unknown as Filed;
    h.papers = { folders: [folder], docs: [doc, book] } as unknown as Papers;

    expect(keysOf(docChoices(book, h))).toEqual(["copyAs", "exportAs", "away"]);
    expect(keysOf(docChoices(doc, h, true))).toEqual(["away", "hide", "drop"]);
  });
});
