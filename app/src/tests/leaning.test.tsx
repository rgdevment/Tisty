import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { fold, unwiden } from "../breadth";
import type { Papers } from "../core";
import { fill, t } from "../locales";
import { sifted } from "../sifting";
import Sidebar from "../ui/Sidebar";
import { asView, type Chosen } from "../views";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
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

const doc = (id: string, title: string, folder: string | null, pageOf?: string) => ({
  id,
  file: `${id}.md`,
  title,
  folder,
  pageOf,
  archived: false,
  away: false,
});

const papers: Papers = {
  folders: [folder("F1", "Trabajo"), folder("F2", "Clientes", "F1"), folder("F3", "Recetas")],
  docs: [
    doc("D1", "Acta de reunión", "F2"),
    doc("D2", "Arroz chileno", "F3"),
    doc("D3", "Anexo de precios", null, "D1"),
  ],
};

const bar = (
  chosen: Chosen = { named: "tasks" },
  onSearch = vi.fn(),
  onHere = vi.fn(),
  onChoose = vi.fn(),
  query = "",
) =>
  render(
    <Sidebar
      lists={[{ id: "L1", name: "Folio Project", order: "a" }]}
      tags={[
        { tag: "web", tasks: 9, docs: 0 },
        { tag: "backend", tasks: 12, docs: 0 },
      ]}
      papers={papers}
      counts={{ tasks: 82, overdue: 9, late: 2, dueToday: 0, upcoming: 10, L1: 8 }}
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
    expect(screen.getByRole("button", { name: "Trabajo" })).toBeTruthy();
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

  it("keeps the list the person narrowed to when a count is opened", () => {
    const onChoose = vi.fn();
    bar({ named: "tasks", lists: ["L1"] }, vi.fn(), vi.fn(), onChoose);
    fireEvent.click(screen.getByText(t("pulseLate")));
    expect(onChoose).toHaveBeenCalledWith({ named: "tasks", slice: "late", lists: ["L1"] });
  });

  it("keeps what was open in the library while tasks have the room", () => {
    bar({ named: "docs" });
    fireEvent.click(screen.getByRole("button", { name: fill("openFolder", "Trabajo") }));
    expect(screen.getByText("Clientes")).toBeTruthy();

    grow();
    fireEvent.click(screen.getByRole("button", { name: t("growLibrary") }));

    expect(screen.getByText("Clientes")).toBeTruthy();
  });

  it("opens the folder picked on the shrunk library, the way to it included", () => {
    bar();
    grow();
    fireEvent.click(screen.getByRole("button", { name: "Trabajo" }));
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

  it("opens a folder of the shrunk library in the library", () => {
    const onHere = vi.fn();
    bar({ named: "tasks" }, vi.fn(), onHere);
    grow();
    fireEvent.click(screen.getByRole("button", { name: "Recetas" }));
    expect(onHere).toHaveBeenCalledWith("F3");
    expect(screen.getByText("Trabajo")).toBeTruthy();
    expect(screen.queryByText("Folio Project")).toBeNull();
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
