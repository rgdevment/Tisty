import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { fold, unwiden } from "../breadth";
import type { Papers } from "../core";
import { capped, numbered } from "../format";
import { lately } from "../lately";
import { listChoices } from "../listMenus";
import { fill, t } from "../locales";
import { sifted } from "../sifting";
import { asksSure } from "../sure";
import Sidebar from "../ui/Sidebar";
import { asView, type Chosen } from "../views";

const ipc = vi.hoisted(() => ({ sent: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.sent.push(cmd);
    return Promise.resolve(null);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const folder = (id: string, name: string, parent: string | null = null) => ({
  id,
  name,
  parent,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
});

const doc = (
  id: string,
  title: string,
  folder: string | null,
  pageOf?: string,
  wrote?: string,
) => ({
  id,
  file: `${id}.md`,
  title,
  folder,
  pageOf,
  wrote,
  archived: false,
  away: false,
});

const papers: Papers = {
  folders: [folder("F1", "Trabajo"), folder("F2", "Clientes", "F1"), folder("F3", "Recetas")],
  docs: [
    doc("D1", "Acta de reunión", "F2"),
    doc("D2", "Arroz chileno", "F3", undefined, new Date().toISOString()),
    doc("D3", "Anexo de precios", null, "D1"),
  ],
};

const bar = (
  chosen: Chosen = { named: "tasks" },
  onSearch = vi.fn(),
  onHere = vi.fn(),
  onChoose = vi.fn(),
  query = "",
  counts: Record<string, number> = {
    tasks: 82,
    overdue: 9,
    late: 2,
    dueToday: 0,
    upcoming: 10,
    L1: 8,
  },
  onListMenu = vi.fn(),
) =>
  render(
    <Sidebar
      lists={[{ id: "L1", name: "Folio Project", order: "a" }]}
      tags={[
        { tag: "web", tasks: 9, docs: 0 },
        { tag: "backend", tasks: 12, docs: 0 },
      ]}
      papers={papers}
      counts={counts}
      chosen={chosen}
      onChoose={onChoose}
      query={query}
      onSearch={onSearch}
      onFile={vi.fn()}
      onHere={onHere}
      onMove={vi.fn()}
      onFolderMenu={vi.fn()}
      onDocMenu={vi.fn()}
      onDocsMenu={vi.fn()}
      onHereMenu={vi.fn()}
      onListMenu={onListMenu}
    />,
  );

const grow = () => fireEvent.click(screen.getByRole("button", { name: t("growTasks") }));

describe("the sidebar gives its room to tasks or to the library, never losing the other", () => {
  afterEach(() => act(() => unwiden()));

  it("keeps the library open and the tasks compact until tasks are given the room", () => {
    bar();
    expect(screen.getByText("Trabajo")).toBeTruthy();
    expect(screen.queryByText("Folio Project")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: t("growTasks") }));

    expect(screen.getByText("Folio Project")).toBeTruthy();
    expect(screen.getByText("#backend")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Trabajo" })).toBeNull();
    expect(screen.getByRole("button", { name: t("shrinkTasks") })).toBeTruthy();
  });

  it("shows the busiest tags first", () => {
    bar();
    grow();
    const said = screen.getAllByText(/^#/).map((one) => one.textContent?.replace(/\d+$/, ""));
    expect(said).toEqual(["#backend", "#web"]);
  });

  it("keeps the search in sight whichever side has the room", () => {
    bar();
    expect(screen.getByRole("textbox", { name: t("sideSearch") })).toBeTruthy();
  });

  it("finds its search with the shortcut, unfolding the rail to reach it", async () => {
    bar();
    act(() => fold(true));
    expect(screen.queryByRole("textbox", { name: t("sideSearch") })).toBeNull();

    fireEvent.keyDown(document.body, { key: "k", ctrlKey: true });
    await act(() => new Promise((done) => requestAnimationFrame(() => done(null))));

    expect(document.activeElement).toBe(screen.getByRole("textbox", { name: t("sideSearch") }));
  });

  it("opens the tasks of the day from its counts", () => {
    const onChoose = vi.fn();
    bar({ named: "docs" }, vi.fn(), vi.fn(), onChoose);
    fireEvent.click(screen.getByText(t("pulseAhead")));
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "upcoming" });
    fireEvent.click(screen.getByText(t("pulseOverdue")));
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "overdue" });
    fireEvent.click(screen.getByText(t("pulseLate")));
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "late" });
  });

  it("opens a count over every list, the narrowing let go and forgotten", () => {
    const onChoose = vi.fn();
    localStorage.setItem("tisty.only", JSON.stringify(["L1"]));
    bar({ named: "tasks", lists: ["L1"] }, vi.fn(), vi.fn(), onChoose);
    fireEvent.click(screen.getByText(t("pulseLate")));
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "late" });
    expect(localStorage.getItem("tisty.only")).toBe("[]");
  });

  it("opens every open task from the views, the narrowing let go, and marks it while it is open", () => {
    const onChoose = vi.fn();
    localStorage.setItem("tisty.only", JSON.stringify(["L1"]));
    bar({ named: "docs" }, vi.fn(), vi.fn(), onChoose, "", { all: 120 });
    const all = screen.getByRole("button", { name: `${t("allTasks")} · 120` });
    expect(all.querySelector("svg")).not.toBeNull();
    expect(all.getAttribute("aria-current")).toBeNull();
    fireEvent.click(all);
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "all" });
    expect(localStorage.getItem("tisty.only")).toBe("[]");
  });

  it("marks every open task as open only while no list narrows it", () => {
    const { unmount } = bar({ named: "tasks", slice: "all" });
    expect(screen.getByRole("button", { name: t("allTasks") }).getAttribute("aria-current")).toBe(
      "page",
    );
    unmount();
    bar({ named: "tasks", slice: "all", lists: ["L1"] });
    expect(
      screen.getByRole("button", { name: t("allTasks") }).getAttribute("aria-current"),
    ).toBeNull();
  });

  it("says no more than +99, the whole number on hover", () => {
    bar({ named: "tasks" }, vi.fn(), vi.fn(), vi.fn(), "", { tasks: 1000, overdue: 100, L1: 99 });
    grow();
    expect(screen.getAllByText("+99").map((one) => one.getAttribute("title"))).toEqual(
      expect.arrayContaining([numbered(1000), "100"]),
    );
    expect(screen.queryByText("1000")).toBeNull();
    expect(screen.getByText("99").getAttribute("title")).toBeNull();
  });

  it("offers a list's own menu from the keyboard, beside its row", () => {
    const onListMenu = vi.fn();
    bar({ named: "tasks" }, vi.fn(), vi.fn(), vi.fn(), "", undefined, onListMenu);
    grow();
    fireEvent.keyDown(screen.getByRole("button", { name: /Folio Project/ }), {
      key: "F10",
      shiftKey: true,
    });
    expect(onListMenu).toHaveBeenCalledWith(expect.objectContaining({ id: "L1" }), {
      x: 24,
      y: 0,
    });
  });

  it("offers a list's own menu from its row", () => {
    const onListMenu = vi.fn();
    bar({ named: "tasks" }, vi.fn(), vi.fn(), vi.fn(), "", undefined, onListMenu);
    grow();
    fireEvent.contextMenu(screen.getByText("Folio Project"), { clientX: 40, clientY: 80 });
    expect(onListMenu).toHaveBeenCalledWith(expect.objectContaining({ id: "L1" }), {
      x: 40,
      y: 80,
    });
  });

  it("folds the library to its title and its strip of counts while tasks have the room", () => {
    bar({ named: "tasks" });
    grow();
    const glance = screen.getByRole("region", { name: t("library") });
    expect(glance.textContent).toContain(t("glanceToday"));
    expect(glance.textContent).toContain(t("glanceFolders"));
    expect(screen.queryByRole("button", { name: /Arroz chileno/ })).toBeNull();
    expect(screen.queryByRole("button", { name: "Recetas" })).toBeNull();
  });

  it("keeps the views in sight while the library has the room", () => {
    bar({ named: "docs" });
    expect(screen.getByRole("navigation", { name: t("tasksViews") })).toBeTruthy();
  });

  it("opens the quick capture from a button, for whoever does not reach for the keyboard", () => {
    bar({ named: "tasks" });
    ipc.sent = [];

    fireEvent.click(screen.getByRole("button", { name: t("captureNew") }));

    expect(ipc.sent).toContain("sow");
  });

  it("keeps what was open in the library while tasks have the room", () => {
    bar({ named: "docs" });
    fireEvent.click(screen.getByRole("button", { name: fill("openFolder", "Trabajo") }));
    expect(screen.getByText("Clientes")).toBeTruthy();

    grow();
    fireEvent.click(screen.getByRole("button", { name: t("growLibrary") }));

    expect(screen.getByText("Clientes")).toBeTruthy();
  });

  it("hands what is typed to the same search the window already runs", () => {
    const onSearch = vi.fn();
    const { unmount } = bar({ named: "tasks" }, onSearch);
    fireEvent.change(screen.getByRole("textbox", { name: t("sideSearch") }), {
      target: { value: "arroz" },
    });
    expect(onSearch).toHaveBeenCalledWith("arroz");
    unmount();

    bar({ named: "search" }, vi.fn(), vi.fn(), vi.fn(), "arroz");
    expect(screen.getByRole("textbox", { name: t("sideSearch") })).toHaveProperty("value", "arroz");
  });

  it("filters the library down to what matches, the folders above it opened", () => {
    bar();
    fireEvent.click(screen.getByRole("button", { name: t("siftDocs") }));
    fireEvent.change(screen.getByRole("textbox", { name: t("siftDocs") }), {
      target: { value: "acta" },
    });
    expect(screen.getByText("Acta de reunión")).toBeTruthy();
    expect(screen.queryByText("Recetas")).toBeNull();

    fireEvent.change(screen.getByRole("textbox", { name: t("siftDocs") }), {
      target: { value: "anexo" },
    });
    expect(screen.getByText("Anexo de precios")).toBeTruthy();

    fireEvent.change(screen.getByRole("textbox", { name: t("siftDocs") }), {
      target: { value: "zzz" },
    });
    expect(screen.getByText(fill("siftNothing", "zzz"))).toBeTruthy();
  });

  it("says the overdue on the rail, and a folder there unfolds into the library", () => {
    const onHere = vi.fn();
    bar({ named: "tasks" }, vi.fn(), onHere);
    grow();
    act(() => fold(true));

    expect(screen.getByRole("button", { name: `${t("tasks")} · 82` }).textContent).toContain("9");
    fireEvent.click(screen.getByRole("button", { name: "Recetas" }));

    expect(onHere).toHaveBeenCalledWith("F3");
    expect(screen.getByText("Trabajo")).toBeTruthy();
    expect(screen.queryByText("Folio Project")).toBeNull();
  });
});

describe("filtering the library", () => {
  it("keeps every folder above a match, and nothing beside it", () => {
    const left = sifted(papers, "acta");
    expect(left.folders.map((one) => one.id)).toEqual(["F1", "F2"]);
    expect(left.docs.map((one) => one.id)).toEqual(["D1"]);
  });

  it("reads past accents and case", () => {
    expect(sifted(papers, "REUNION").docs.map((one) => one.id)).toEqual(["D1"]);
  });

  it("brings the document a matching page hangs from", () => {
    const left = sifted(papers, "anexo");
    expect(left.docs.map((one) => one.id)).toEqual(["D1", "D3"]);
    expect(left.folders.map((one) => one.id)).toEqual(["F1", "F2"]);
  });

  it("keeps a folder whose own name matches, with all it holds", () => {
    const left = sifted(papers, "recetas");
    expect(left.folders.map((one) => one.id)).toEqual(["F3"]);
    expect(left.docs.map((one) => one.id)).toEqual(["D2"]);

    const work = sifted(papers, "trabajo");
    expect(work.folders.map((one) => one.id)).toEqual(["F1", "F2"]);
    expect(work.docs.map((one) => one.id)).toEqual(["D1"]);
  });
});

describe("the day's counts that point somewhere of their own", () => {
  it("asks for the overdue and the late apart from the rest of today", () => {
    expect(asView({ named: "tasks", slice: "overdue" }).window).toBe("overdue");
    expect(asView({ named: "tasks", slice: "late" }).window).toBe("late");
    expect(asView({ named: "tasks", slice: "today" }).window).toBe("today");
  });
});

describe("what the shrunk library says at a glance", () => {
  const now = new Date(2026, 10, 10, 15, 0);
  const daysAgo = (days: number) => new Date(2026, 10, 10 - days, 9, 0).toISOString();

  it("counts what was written today and this week", () => {
    const seen = lately(
      {
        folders: [folder("F1", "Trabajo"), { ...folder("F9", "Viejo"), away: true }],
        docs: [
          doc("A", "Hoy", "F1", undefined, daysAgo(0)),
          doc("B", "Ayer", "F1", undefined, daysAgo(1)),
          doc("C", "Hace seis días", null, undefined, daysAgo(6)),
          doc("D", "Hace una semana", null, undefined, daysAgo(7)),
          doc("E", "Sin fecha", null),
        ],
      },
      now,
    );
    expect(seen.today).toBe(1);
    expect(seen.week).toBe(3);
    expect(seen.folders).toBe(1);
  });

  it("counts nothing written on a day still to come", () => {
    const seen = lately(
      { folders: [], docs: [doc("A", "Mañana", null, undefined, daysAgo(-1))] },
      now,
    );
    expect(seen.today).toBe(0);
    expect(seen.week).toBe(0);
  });

  it("counts the marked, and leaves pages and the archive out", () => {
    const marked = { at: daysAgo(0), said: "agent" };
    const seen = lately(
      {
        folders: [],
        docs: [
          { ...doc("A", "Marcado", null, undefined, daysAgo(0)), flagged: marked },
          { ...doc("B", "Archivado", null, undefined, daysAgo(0)), away: true, flagged: marked },
          doc("C", "Página", null, "A", daysAgo(0)),
        ],
      },
      now,
    );
    expect(seen.marked).toBe(1);
    expect(seen.today).toBe(1);
  });
});

describe("a count past what fits", () => {
  it("stops at +99", () => {
    expect([capped(0), capped(99), capped(100), capped(5000)]).toEqual(["0", "99", "+99", "+99"]);
  });
});

describe("a list's own menu", () => {
  const hands = { edit: vi.fn(), gone: vi.fn(), fail: vi.fn() };
  const list = { id: "L1", name: "Salud", order: "a" };
  const pick = (key: string) =>
    listChoices(list, 0, hands)
      .find((one) => one.key === key)
      ?.onPick?.();

  afterEach(() => {
    asksSure(null);
    vi.clearAllMocks();
  });

  it("opens the list's own window to change its name, icon and colour", () => {
    const edit = listChoices(list, 0, hands).find((one) => one.key === "edit");
    edit?.onPick?.();
    expect(hands.edit).toHaveBeenCalledWith(list);
  });

  it("offers to archive only a list with nothing left in it", () => {
    const off = (held: number) =>
      listChoices(list, held, hands).find((one) => one.key === "archive")?.off;
    expect(off(0)).toBe(false);
    expect(off(3)).toBe(true);
  });

  it("leaves the list once it is archived, so nothing stays open on it", async () => {
    pick("archive");
    await vi.waitFor(() => expect(hands.gone).toHaveBeenCalledWith("L1"));
  });

  it("leaves the list once it is deleted, and stays when the deletion is let go", async () => {
    asksSure(() => Promise.resolve(false));
    pick("drop");
    await act(() => Promise.resolve());
    expect(hands.gone).not.toHaveBeenCalled();

    asksSure(() => Promise.resolve(true));
    pick("drop");
    await vi.waitFor(() => expect(hands.gone).toHaveBeenCalledWith("L1"));
  });
});
