import { fireEvent, render, renderHook, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Marked, Task } from "../core";
import { fireAll, useHiddenGoing } from "../dropLater";
import { dayOf } from "../format";
import { asideWords, letAsideGo, setAside } from "../keptAside";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { cut, kept, unansweredWords } from "../saving";
import { asksSure } from "../sure";
import { alike, dropWord } from "../tidying";
import { tongueName } from "../tongues";
import Docs from "../ui/Docs";
import Tree from "../ui/Tree";
import { usePapers } from "../usePapers";
import { knowAgents } from "../who";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  tasks: {} as Record<string, unknown>,
  docs: [] as unknown[],
  gone: [] as string[],
  last: true,
  dropping: null as ((id: string) => void) | null,
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    switch (cmd) {
      case "doc_read":
        return Promise.resolve("# Notes\n\nwords");
      case "doc_backable":
        return Promise.resolve(true);
      case "doc_compacted_last":
        return Promise.resolve(ipc.last);
      case "doc_back":
        return Promise.resolve({ id: args.id, title: "Notes" });
      case "docs":
        return Promise.resolve({ folders: [], docs: ipc.docs });
      case "doc_drop": {
        const id = String(args.id);
        ipc.dropping?.(id);
        if (ipc.gone.includes(id)) return Promise.reject({ code: "noSuchDoc" });
        const one = (ipc.docs as Filed[]).find((kept) => kept.id === id);
        const anchor = one?.tongue?.[0];
        const kin = (ipc.docs as Filed[]).filter(
          (kept) => anchor && kept.tongue?.[0] === anchor && kept.folder === one?.folder,
        );
        ipc.gone.push(id, ...kin.map((kept) => kept.id));
        return Promise.resolve(null);
      }
      case "task_of": {
        const one = ipc.tasks[String(args.id)];
        return one ? Promise.resolve(one) : Promise.reject({ code: "noSuchTask" });
      }
      default:
        return Promise.resolve(null);
    }
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(true),
  save: () => Promise.resolve(null),
  open: () => Promise.resolve(null),
}));

vi.mock("../ui/Sheet", () => ({
  default: ({ value }: { value: string }) => (
    <textarea aria-label="editor" value={value} readOnly />
  ),
}));

const AT = "2026-10-19T10:00:00Z";
const TASK = "01JB2Q7X3Y4Z5A6B7C8D9E0F1G";

const doc = (one: Partial<Filed> & { id: string; file: string }): Filed => ({
  title: "Notes",
  folder: null,
  archived: false,
  away: false,
  ...one,
});

const marked = (over: Partial<Marked> = {}): Marked => ({
  at: AT,
  said: "Same text as the original.",
  ...over,
});

const called = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const show = (known: Filed[], more: Partial<Parameters<typeof Docs>[0]> = {}) => {
  const hands = {
    onDoc: vi.fn(),
    onTask: vi.fn(),
    onDrop: vi.fn(),
    onDropAll: vi.fn(),
    onKept: vi.fn(),
    ...more,
  };
  render(<Docs open="d-1" known={known} onError={vi.fn()} {...hands} />);
  return hands;
};

beforeEach(() => {
  ipc.calls = [];
  ipc.last = true;
  ipc.tasks = {};
  ipc.docs = [];
  ipc.gone = [];
  ipc.dropping = null;
  asksSure(null);
  knowAgents({}, { clients: { "claude-code": "Claude Code" } });
});

describe("a mark that says why", () => {
  it("names the copy it repeats, and opens that one from the mark", async () => {
    const original = doc({ id: "D2", file: "d-2", title: "Handbook" });
    const own = doc({
      id: "D1",
      file: "d-1",
      flagged: marked({ reason: "duplicate_of", instead: "d-2", via: "claude-code" }),
    });
    const { onDoc, onDrop } = show([own, original]);

    const link = await screen.findByRole("button", { name: "Handbook" });
    expect(link.parentElement?.textContent).toContain("A copy of «Handbook»");
    expect(screen.getByText(fill("docFlaggedBy", "Claude Code"))).toBeTruthy();
    expect(screen.getByText("Same text as the original.")).toBeTruthy();

    await userEvent.click(link);
    expect(onDoc).toHaveBeenCalledWith("d-2");

    await userEvent.click(screen.getByRole("button", { name: t("flagDropCopy") }));
    expect(onDrop).toHaveBeenCalledWith(own);
  });

  it("opens the task that replaced it", async () => {
    const task = { id: TASK, title: "Ship the release", list: "L1" } as Task;
    ipc.tasks[TASK] = task;
    show([
      doc({ id: "D1", file: "d-1", flagged: marked({ reason: "superseded", instead: TASK }) }),
    ]);

    const link = await screen.findByRole("button", { name: "Ship the release" });
    expect(link.parentElement?.textContent).toContain("Replaced by «Ship the release»");
  });

  it("hands the task it names to whoever opens tasks", async () => {
    const task = { id: TASK, title: "Ship the release", list: "L1" } as Task;
    ipc.tasks[TASK] = task;
    const { onTask } = show([
      doc({ id: "D1", file: "d-1", flagged: marked({ reason: "moved", instead: TASK }) }),
    ]);

    await userEvent.click(await screen.findByRole("button", { name: "Ship the release" }));
    expect(onTask).toHaveBeenCalledWith(task);
  });

  it("says plainly when what it points at is no longer there", async () => {
    show([
      doc({ id: "D1", file: "d-1", flagged: marked({ reason: "duplicate_of", instead: "d-9" }) }),
    ]);

    expect(
      await screen.findByText(`${fill("flagCopyOf", t("flagNoLonger"))}`, { exact: false }),
    ).toBeTruthy();
  });

  it("says wrong in words and offers to delete it", async () => {
    show([doc({ id: "D1", file: "d-1", flagged: marked({ reason: "wrong" }) })]);

    expect(await screen.findByText(t("flagWrong"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("flagDropIt") })).toBeTruthy();
  });

  it("reads a reason it does not know as a plain mark", async () => {
    show([doc({ id: "D1", file: "d-1", flagged: marked({ reason: "someday" }) })]);

    expect(await screen.findByText(t("docFlagged"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("flagDropIt") })).toBeTruthy();
  });

  it("takes the mark off with the existing unflag", async () => {
    const { onKept } = show([
      doc({ id: "D1", file: "d-1", flagged: marked({ reason: "scratch" }) }),
    ]);

    await userEvent.click(await screen.findByRole("button", { name: t("unflagIt") }));
    await waitFor(() => expect(onKept).toHaveBeenCalled());
    expect(called("doc_unflag").map((one) => one.args.id)).toEqual(["D1"]);
  });
});

describe("a set marked in one go", () => {
  const set = () => [
    doc({ id: "D1", file: "d-1", title: "Draft A", flagged: marked({ reason: "scratch" }) }),
    doc({ id: "D2", file: "d-2", title: "Draft B", flagged: marked({ reason: "scratch" }) }),
    doc({ id: "D3", file: "d-3", title: "Draft C", flagged: marked({ reason: "scratch" }) }),
    doc({ id: "D4", file: "d-4", title: "Other", flagged: marked({ reason: "wrong" }) }),
  ];

  it("offers to delete every one of the set", async () => {
    const known = set();
    const { onDropAll } = show(known);

    await userEvent.click(await screen.findByRole("button", { name: fill("flagDropAll", "3") }));
    expect(onDropAll).toHaveBeenCalledWith(known.slice(0, 3));
  });

  it("takes every mark off once the person agrees to the titles", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const { onKept } = show(set());

    await userEvent.click(await screen.findByRole("button", { name: fill("flagUnmarkAll", "3") }));
    await waitFor(() => expect(onKept).toHaveBeenCalled());
    expect(asked[0]).toContain("· Draft A\n· Draft B\n· Draft C");
    expect(called("doc_unflag").map((one) => one.args.id)).toEqual(["D1", "D2", "D3"]);
  });

  it("leaves the marks when the person says no", async () => {
    asksSure(() => Promise.resolve(false));
    show(set());

    await userEvent.click(await screen.findByRole("button", { name: fill("flagUnmarkAll", "3") }));
    await new Promise((settle) => setTimeout(settle, 0));
    expect(called("doc_unflag")).toEqual([]);
  });

  it("offers nothing for a set of one", async () => {
    show(
      set()
        .slice(3)
        .map((one) => ({ ...one, id: "D1", file: "d-1" })),
    );

    await screen.findByText(t("flagWrong"));
    expect(screen.queryByRole("button", { name: /Unmark all/ })).toBeNull();
  });

  it("knows a set by when, what, why and instead, and leaves archived ones out", () => {
    const own = doc({ id: "D1", file: "d-1", flagged: marked({ reason: "moved", instead: "x" }) });
    const twin = doc({ id: "D2", file: "d-2", flagged: marked({ reason: "moved", instead: "x" }) });
    const elsewhere = doc({
      id: "D3",
      file: "d-3",
      flagged: marked({ reason: "moved", instead: "y" }),
    });
    const later = doc({
      id: "D4",
      file: "d-4",
      flagged: marked({ reason: "moved", instead: "x", at: "2026-10-20T10:00:00Z" }),
    });
    const shelved = { ...twin, id: "D5", file: "d-5", away: true };

    expect(alike([own, twin, elsewhere, later, shelved], own)).toEqual([own, twin]);
  });

  it("calls the delete by what the mark said", () => {
    expect(dropWord("duplicate_of")).toBe("flagDropCopy");
    for (const other of ["moved", "superseded", "scratch", "wrong", "obsolete", "someday"]) {
      expect(dropWord(other)).toBe("flagDropIt");
    }
    expect(dropWord(undefined)).toBe("flagDropIt");
  });
});

describe("deleting a set", () => {
  const goneNow = async () => {
    const before = called("docs").length;
    fireAll();
    await waitFor(() => expect(called("docs").length).toBeGreaterThan(before));
  };

  const hands = () => ({
    chosen: { named: "docs" as const, doc: "d-2" },
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

  it("asks once naming each document and the pages that go with it", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const all = [
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B", pageOf: "D1" }),
      doc({ id: "D3", file: "d-3", title: "Draft C" }),
    ];
    const given = hands();
    const { result } = renderHook(() => usePapers(given));

    await result.current.dropDocs(all);

    expect(asked).toEqual([`${fill("dropManySure", "2")}\n\n· Draft A\n    · Draft B\n· Draft C`]);
    expect(called("doc_drop")).toEqual([]);
    expect(given.setChosen).toHaveBeenCalledWith({ named: "docs" });
    await goneNow();
    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1", "D3"]);
  });

  it("lets go of the words a save left behind for each document and page that went", async () => {
    asksSure(() => Promise.resolve(true));
    for (const file of ["d-1", "d-2", "d-9"]) {
      await kept(file, `palabras de ${file}`, () =>
        Promise.reject({ code: "documentMoved" }),
      ).catch(() => {});
    }
    const { result } = renderHook(() => usePapers(hands()));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B", pageOf: "D1" }),
    ]);
    await goneNow();

    expect(unansweredWords("d-1")).toBeUndefined();
    expect(unansweredWords("d-2")).toBeUndefined();
    expect(unansweredWords("d-9")).toBe("palabras de d-9");
    cut("d-9");
  });

  it("says this document and the page it holds when there is one of each", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const { result } = renderHook(() => usePapers(hands()));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B", pageOf: "D1" }),
    ]);

    expect(asked).toEqual([fill("dropPagesSure", "Draft A")]);
    await goneNow();
    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1"]);
  });

  it("asks with the plain titles when no page goes along", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const { result } = renderHook(() => usePapers(hands()));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D3", file: "d-3", title: "Draft C" }),
    ]);

    expect(asked).toEqual([`${fill("dropManySure", "2")}\n\n· Draft A\n· Draft C`]);
    await goneNow();
  });

  it("names and lets go of every language the store deletes with a document, the archived one too", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    ipc.docs = [
      doc({ id: "D1", file: "d-1", lang: "en", tongue: ["D1", 0] }),
      doc({ id: "D2", file: "d-2", lang: "es", tongue: ["D1", 1], archived: true, away: true }),
    ];
    await kept("d-2", "palabras de d-2", () => Promise.reject({ code: "documentMoved" })).catch(
      () => {},
    );
    const { result } = renderHook(() => usePapers(hands()));
    await waitFor(() => expect(result.current.papers.docs).toHaveLength(2));

    await result.current.dropDoc(ipc.docs[0] as Filed);
    await goneNow();

    expect(asked).toEqual([
      fill("dropTonguesSure", "Notes", [tongueName("en"), tongueName("es")].join(", ")),
    ]);
    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1"]);
    expect(unansweredWords("d-2")).toBeUndefined();
  });

  it("deletes the languages of one document once, names them, and still deletes the rest", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    ipc.docs = [
      doc({ id: "D1", file: "d-1", title: "Notes", lang: "en", tongue: ["D1", 0] }),
      doc({ id: "D2", file: "d-2", title: "Notas", lang: "es", tongue: ["D1", 1] }),
      doc({ id: "D3", file: "d-3", title: "Draft C" }),
    ];
    const given = hands();
    const { result } = renderHook(() => usePapers(given));
    await waitFor(() => expect(result.current.papers.docs).toHaveLength(3));

    await result.current.dropDocs(ipc.docs as Filed[]);
    await goneNow();

    const both = [tongueName("en"), tongueName("es")].join(", ");
    expect(asked).toEqual([`${fill("dropManySure", "2")}\n\n· Notes (${both})\n· Draft C`]);
    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1", "D3"]);
    expect(given.setError).not.toHaveBeenCalled();
  });

  it("takes a page of another language with its document, whichever comes first in the set", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    ipc.docs = [
      doc({ id: "D1", file: "d-1", title: "Notes", lang: "en", tongue: ["D1", 0] }),
      doc({ id: "D2", file: "d-2", title: "Notas", lang: "es", tongue: ["D1", 1] }),
      doc({ id: "P2", file: "p-2", title: "Anexo", pageOf: "D2" }),
    ];
    const { result } = renderHook(() => usePapers(hands()));
    await waitFor(() => expect(result.current.papers.docs).toHaveLength(3));

    const [first, , page] = ipc.docs as Filed[];
    await result.current.dropDocs([page, first]);
    await goneNow();

    const both = [tongueName("en"), tongueName("es")].join(", ");
    expect(asked).toEqual([fill("dropTonguesSure", "Notes", both)]);
    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1"]);
  });

  it("goes on with the rest of the set when one of it is refused, and says why", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.gone = ["D1"];
    const given = hands();
    const { result } = renderHook(() => usePapers(given));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D3", file: "d-3", title: "Draft C" }),
    ]);
    await goneNow();

    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1", "D3"]);
    expect(given.setError).toHaveBeenCalledTimes(1);
  });

  it("names every refusal, and closes the open page when the delete is confirmed, not when it runs", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.gone = ["D1", "D2"];
    const given = hands();
    const { result } = renderHook(() => usePapers(given));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B" }),
      doc({ id: "D3", file: "d-3", title: "Draft C", folder: "F1" }),
    ]);

    expect(called("doc_drop")).toEqual([]);
    expect(given.setChosen).toHaveBeenCalledWith({ named: "docs" });
    expect(given.setReturning).toHaveBeenCalledWith("unfiled");
    await goneNow();

    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D1", "D2", "D3"]);
    const why = saidPlainly({ code: "noSuchDoc" });
    expect(given.setError).toHaveBeenCalledWith(
      `${fill("dropRefused", "Draft A", why)} · ${fill("dropRefused", "Draft B", why)}`,
    );
  });

  it("leaves the open document as soon as the delete is confirmed, before any of it runs", async () => {
    asksSure(() => Promise.resolve(true));
    const given = { ...hands(), chosen: { named: "docs" as const, doc: "d-1" } };
    const answered: string[] = [];
    const drops = () => called("doc_drop").length;
    given.setChosen.mockImplementation(() => answered.push(`left after ${drops()}`));
    given.setReturning.mockImplementation(() => answered.push(`focus after ${drops()}`));
    const { result } = renderHook(() => usePapers(given));

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B" }),
    ]);

    expect(answered).toEqual(["left after 0", "focus after 0"]);
    expect(given.setChosen).toHaveBeenCalledWith({ named: "docs" });
    await goneNow();
  });

  it("leaves the page that is open when the delete is confirmed, whichever was open before", async () => {
    asksSure(() => Promise.resolve(true));
    const given = { ...hands(), chosen: { named: "docs" as const, doc: "d-1" } };
    const { result, rerender } = renderHook((chosen) => usePapers({ ...given, chosen }), {
      initialProps: given.chosen,
    });
    rerender({ named: "docs", doc: "d-9" });

    await result.current.dropDocs([
      doc({ id: "D1", file: "d-1", title: "Draft A" }),
      doc({ id: "D2", file: "d-2", title: "Draft B" }),
    ]);
    await goneNow();

    expect(given.setChosen).not.toHaveBeenCalled();
  });

  it("shows a document deleted alone again, and says why, when its delete is refused", async () => {
    asksSure(() => Promise.resolve(true));
    ipc.gone = ["D2"];
    const given = hands();
    const { result } = renderHook(() => usePapers(given));
    const hidden = renderHook(() => useHiddenGoing());

    await result.current.dropDoc(doc({ id: "D2", file: "d-2", title: "Draft B" }));

    expect(hidden.result.current.has("D2")).toBe(true);
    expect(called("doc_drop")).toEqual([]);
    await goneNow();

    expect(called("doc_drop").map((one) => one.args.id)).toEqual(["D2"]);
    expect(hidden.result.current.has("D2")).toBe(false);
    expect(given.setError).toHaveBeenCalledWith(saidPlainly({ code: "noSuchDoc" }));
  });

  it("lets go of the words kept aside for each document that went, and only those", async () => {
    asksSure(() => Promise.resolve(true));
    setAside("d-1", "palabras de d-1");
    setAside("d-9", "palabras de d-9");
    const { result } = renderHook(() => usePapers(hands()));

    await result.current.dropDoc(doc({ id: "D1", file: "d-1", title: "Draft A" }));
    await goneNow();

    expect(asideWords("d-1")).toBeUndefined();
    expect(asideWords("d-9")).toBe("palabras de d-9");
    letAsideGo("d-9");
  });

  it("deletes nothing when the person says no", async () => {
    asksSure(() => Promise.resolve(false));
    const { result } = renderHook(() => usePapers(hands()));

    await result.current.dropDocs([doc({ id: "D1", file: "d-1" })]);

    expect(called("doc_drop")).toEqual([]);
  });
});

describe("a document an assistant compacted", () => {
  const compacted = (last = true) => {
    ipc.last = last;
    return doc({ id: "D1", file: "d-1", compacted: { at: AT, via: "claude-code" } });
  };

  it("says who compacted it and when", async () => {
    show([compacted()]);

    expect(await screen.findByText(fill("docCompactedBy", "Claude Code", dayOf(AT)))).toBeTruthy();
    expect(called("doc_compacted_last").map((one) => one.args.id)).toContain("d-1");
  });

  it("names an assistant when it did not say its name", async () => {
    show([doc({ id: "D1", file: "d-1", compacted: { at: AT } })]);

    expect(
      await screen.findByText(fill("docCompactedBy", t("anAssistant"), dayOf(AT))),
    ).toBeTruthy();
  });

  it("asks before undoing, since a later edit of the person's would be what goes back", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const { onKept } = show([compacted()]);

    await userEvent.click(await screen.findByRole("button", { name: t("compactUndo") }));
    await waitFor(() => expect(onKept).toHaveBeenCalled());
    expect(called("doc_back").map((one) => one.args.id)).toEqual(["d-1"]);
    expect(asked).toEqual([`${t("backSure")} ${t("typedSinceGoes")}`]);
  });

  it("offers no undo once something was written after it", async () => {
    show([compacted(false)]);

    expect(
      await screen.findByText(fill("docCompactedSince", "Claude Code", dayOf(AT))),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: t("compactKeep") })).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("compactUndo") })).toBeNull();
  });

  it("keeps it and lets the mark go", async () => {
    const { onKept } = show([compacted()]);

    await userEvent.click(await screen.findByRole("button", { name: t("compactKeep") }));
    await waitFor(() => expect(onKept).toHaveBeenCalled());
    expect(called("doc_keep_compacted").map((one) => one.args.id)).toEqual(["D1"]);
  });

  it("marks its row in the tree while it stays compacted", () => {
    render(
      <Tree
        papers={{
          folders: [],
          docs: [compacted(), doc({ id: "D2", file: "d-2", title: "Plain" })],
        }}
        onOpen={vi.fn()}
        onFile={vi.fn()}
      />,
    );
    for (let round = 0; round < 4; round += 1) {
      for (const one of screen.queryAllByRole("button", { name: /^Open / })) fireEvent.click(one);
    }

    const row = screen.getByRole("button", { name: "Notes" });
    expect(within(row).getByTitle(fill("docCompactedBy", "Claude Code", dayOf(AT)))).toBeTruthy();
    expect(within(screen.getByRole("button", { name: "Plain" })).queryByText("◇")).toBeNull();
  });
});
