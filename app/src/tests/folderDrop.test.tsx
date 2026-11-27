import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_ICON, AGENTS_NAME } from "../agentsSync";
import type { Filed, Folded } from "../core";
import { fireAll, UNDO_FOR, useGoing, useHiddenGoing } from "../dropLater";
import { asksFolderDrop, type FolderAnswer, type FolderAsked } from "../folderDropping";
import { fill, t } from "../locales";
import { asksSure } from "../sure";
import { usePapers } from "../usePapers";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  refuses: false,
  board: null as string | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "docs") return Promise.resolve({ folders: FOLDERS, docs: DOCS });
    if (cmd === "brainstorm_of")
      return Promise.resolve(ipc.board && { file: ipc.board, title: "Ideas", id: "d3" });
    if (cmd === "folder_drop" && ipc.refuses) return Promise.reject({ code: "noSuchFolder" });
    return Promise.resolve(null);
  },
}));

const folder = (id: string, name: string, parent: string | null = null): Folded => ({
  id,
  name,
  parent,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
});

const doc = (id: string, title: string, at: string | null): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: at,
  archived: false,
  away: false,
});

const FOLDERS = [
  folder("up", "Oficina"),
  folder("work", "Trabajo", "up"),
  folder("sub", "Dentro", "work"),
  folder("empty", "Vacia"),
  { ...folder("ag", AGENTS_NAME), icon: AGENTS_ICON },
  folder("agsub", "Global", "ag"),
  folder("shut", "Cerrada"),
  folder("boxed", "Caja"),
  { ...folder("shelf", "Estante", "boxed"), away: true, archived: true },
];
const DOCS = [
  doc("d1", "Acta", "work"),
  doc("d2", "Plan", "sub"),
  doc("d3", "Suelto", null),
  doc("d4", "Regla", "agsub"),
  { ...doc("book", "Diario 2026", null), diary: true, away: true },
  { ...doc("d5", "Secreto", "shut"), locked: true },
  { ...doc("d6", "Guardado", "shelf"), away: true },
];

const hands = () => ({
  chosen: { named: "docs" as const, doc: "f-d2" },
  setChosen: vi.fn(),
  here: undefined,
  setHere: vi.fn(),
  setReturning: vi.fn(),
  setBacking: vi.fn(),
  setError: vi.fn(),
  openTask: vi.fn(),
  noted: vi.fn(),
  lookForAStar: vi.fn(),
  carries: { current: null },
  paging: { current: null },
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).map((one) => one.args);

const loaded = async (given = hands()) => {
  const hook = renderHook(() => usePapers(given));
  await waitFor(() => expect(hook.result.current.papers.folders).toHaveLength(FOLDERS.length));
  return { hook, given };
};

const asked: FolderAsked[] = [];
const answering = (answer: FolderAnswer | null) =>
  asksFolderDrop((one) => {
    asked.push(one);
    return Promise.resolve(answer);
  });

beforeEach(() => {
  ipc.calls = [];
  ipc.refuses = false;
  ipc.board = null;
  asked.length = 0;
  asksSure(null);
});

afterEach(async () => {
  asksFolderDrop(null);
  asksSure(null);
  fireAll();
  await act(() => Promise.resolve());
  vi.useRealTimers();
});

describe("deleting a folder that holds something", () => {
  it("asks what the folder holds, with the folder above, before anything goes", async () => {
    answering(null);
    const { hook } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));

    expect(asked).toHaveLength(1);
    expect(asked[0]).toMatchObject({ docs: 1, folders: 1, handsOnly: false });
    expect(asked[0].up?.id).toBe("up");
    expect(sent("folder_drop")).toEqual([]);
  });

  it("does nothing when the question is cancelled", async () => {
    answering(null);
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    fireAll();
    await act(() => Promise.resolve());

    expect(sent("folder_drop")).toEqual([]);
    expect(given.setChosen).not.toHaveBeenCalled();
  });

  it("sends the chosen contents once the undo time is over, and not before", async () => {
    answering({ contents: "into", into: "empty" });
    const { hook } = await loaded();
    vi.useFakeTimers();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    await act(() => vi.advanceTimersByTimeAsync(UNDO_FOR - 1));
    expect(sent("folder_drop")).toEqual([]);
    await act(() => vi.advanceTimersByTimeAsync(1));

    expect(sent("folder_drop")).toEqual([{ id: "work", contents: "into", into: "empty" }]);
  });

  it("sends «up» with no folder picked", async () => {
    answering({ contents: "up" });
    const { hook } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    await act(async () => fireAll());

    expect(sent("folder_drop")).toEqual([{ id: "work", contents: "up", into: undefined }]);
  });

  it("hides only the folder when its contents move, and all of it when they go too", async () => {
    answering({ contents: "up" });
    const { hook } = await loaded();
    const hidden = renderHook(() => useHiddenGoing());

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    expect([...hidden.result.current]).toEqual(["work"]);
    await act(async () => fireAll());

    answering({ contents: "drop" });
    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    expect([...hidden.result.current].sort()).toEqual(["d1", "d2", "sub", "work"]);
  });

  it("closes the open page when it is inside what goes with the folder, as the delete is confirmed", async () => {
    answering({ contents: "drop" });
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));

    expect(given.setChosen).toHaveBeenCalledWith({ named: "docs" });
    expect(given.setReturning).toHaveBeenCalledWith("up");
    expect(sent("folder_drop")).toEqual([]);
  });

  it("keeps the open page when what is inside only moves", async () => {
    answering({ contents: "up" });
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));

    expect(given.setChosen).not.toHaveBeenCalled();
  });

  it("says why when the core refuses, and shows the folder again", async () => {
    ipc.refuses = true;
    answering({ contents: "up" });
    const { hook, given } = await loaded();
    const hidden = renderHook(() => useHiddenGoing());

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));
    expect(hidden.result.current.has("work")).toBe(true);
    await act(async () => fireAll());

    await waitFor(() => expect(given.setError).toHaveBeenCalledTimes(1));
    expect(hidden.result.current.has("work")).toBe(false);
  });
});

describe("deleting a folder with nothing in it", () => {
  it("asks the plain question and never the one about its contents", async () => {
    const plain: string[] = [];
    asksSure((said) => {
      plain.push(said);
      return Promise.resolve(true);
    });
    answering({ contents: "drop" });
    const { hook } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[3]));
    await act(async () => fireAll());

    expect(plain).toEqual([fill("dropFolderSure", "Vacia")]);
    expect(asked).toEqual([]);
    expect(sent("folder_drop")).toEqual([{ id: "empty", contents: "up", into: undefined }]);
  });

  it("deletes nothing when the person says no", async () => {
    asksSure(() => Promise.resolve(false));
    const { hook } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[3]));
    await act(async () => fireAll());

    expect(sent("folder_drop")).toEqual([]);
  });
});

describe("deleting a folder of Agents that still holds files", () => {
  it("says so at once, without asking or sending anything", async () => {
    answering({ contents: "drop" });
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[5]));
    await act(async () => fireAll());

    expect(given.setError).toHaveBeenCalledWith(t("agentsFolderHolds"));
    expect(asked).toEqual([]);
    expect(sent("folder_drop")).toEqual([]);
  });
});

describe("bringing a diary back from the archive", () => {
  it("goes straight back to where it was, without opening the bring-back dialog", async () => {
    const { hook, given } = await loaded();

    await act(async () => hook.result.current.bringBack(DOCS[4]));

    await waitFor(() => expect(sent("doc_away")).toEqual([{ id: "book", away: false }]));
    expect(given.setBacking).not.toHaveBeenCalled();
  });
});

describe("deleting what cannot go", () => {
  it("says a locked document is locked, without asking or sending anything", async () => {
    const { hook, given } = await loaded();
    const locked = { ...DOCS[0], locked: true };

    await act(() => hook.result.current.dropDoc(locked));
    await act(async () => fireAll());

    expect(given.setError).toHaveBeenCalledWith(t("lockedSaid"));
    expect(sent("doc_drop")).toEqual([]);
  });

  it("says the Ideas sheet stays, without asking or sending anything", async () => {
    ipc.board = DOCS[2].file;
    const { hook, given } = await loaded();
    await waitFor(() => expect(hook.result.current.board).toBe(DOCS[2].file));

    await act(() => hook.result.current.dropDoc(DOCS[2]));
    await act(async () => fireAll());

    expect(given.setError).toHaveBeenCalledWith(t("ideasStay"));
    expect(sent("doc_drop")).toEqual([]);
  });

  it("lets the others of a set go and says why one stays", async () => {
    asksSure(() => Promise.resolve(true));
    const { hook, given } = await loaded();
    const locked = { ...DOCS[0], locked: true };

    await act(() => hook.result.current.dropDocs([locked, DOCS[2]]));
    await act(async () => fireAll());

    expect(given.setError).toHaveBeenCalledWith(t("lockedSaid"));
    expect(sent("doc_drop").map((one) => one.id)).toEqual(["d3"]);
  });
});

describe("a folder deleted with its contents going up", () => {
  it("tells the sidebar where the contents will land while it waits", async () => {
    answering({ contents: "up" });
    const { hook } = await loaded();
    const going = renderHook(() => useGoing());

    await act(() => hook.result.current.dropFolder(FOLDERS[1]));

    expect(going.result.current.map((one) => one.moves)).toEqual([{ from: "work", to: "up" }]);
  });
});

describe("deleting a folder with a locked document inside", () => {
  it("says so at once when its contents would go with it, and nothing waits to go", async () => {
    answering({ contents: "drop" });
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[6]));

    expect(given.setError).toHaveBeenCalledWith(fill("folderKeepsLocked", "Secreto"));
    await act(async () => fireAll());
    expect(sent("folder_drop")).toEqual([]);
  });
});

describe("deleting a folder that holds an archived one", () => {
  it("says the archive keeps it at once, and nothing waits to go", async () => {
    answering({ contents: "drop" });
    const { hook, given } = await loaded();

    await act(() => hook.result.current.dropFolder(FOLDERS[7]));

    expect(given.setError).toHaveBeenCalledWith(t("folderIsAway"));
    await act(async () => fireAll());
    expect(sent("folder_drop")).toEqual([]);
  });
});

describe("the question before several picked documents go", () => {
  it("names the undo time instead of saying it cannot be undone", async () => {
    const said: string[] = [];
    asksSure((text) => {
      said.push(text);
      return Promise.resolve(false);
    });
    const { hook } = await loaded();

    await act(() => hook.result.current.dropDocs([DOCS[0], DOCS[2]]));

    expect(said[0]?.startsWith(fill("dropManySure", "2"))).toBe(true);
    expect(sent("doc_drop")).toEqual([]);
  });
});
