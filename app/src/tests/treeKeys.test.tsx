import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentRow } from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
import { showAll } from "../hiding";
import { fill, t } from "../locales";
import { placesRules } from "../places";
import Tree from "../ui/Tree";

const ruling = vi.hoisted(() => {
  const open = { makes: ["doc"], pages: true, moves: true, copies: true };
  return {
    plain: open,
    agents: open,
    ideas: { makes: [], pages: false, moves: false, copies: false },
    diary: { makes: [], pages: false, moves: false, copies: true },
  };
});

const rows = vi.hoisted(() => ({ answer: [] as unknown[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "places_rules") return Promise.resolve(ruling);
    if (cmd === "agents_rows") return Promise.resolve(rows.answer);
    if (cmd === "icons") return Promise.resolve([]);
    return Promise.resolve(null);
  },
}));

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
  ...more,
});

const paper = (id: string, title: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const names = ["Uno", "Dos", "Tres", "Cuatro"];
const loose = names.map((name, at) => paper(`d${at + 1}`, name));
const casa = folder("f1", "Casa");
const papers: Papers = { folders: [casa], docs: loose };

beforeAll(async () => {
  document.elementFromPoint = () => null;
  await placesRules();
});

beforeEach(() => {
  showAll();
  rows.answer = [];
});

const draw = (given: Papers = papers, props: Partial<Parameters<typeof Tree>[0]> = {}) =>
  render(<Tree papers={given} unfolded onOpen={vi.fn()} onFile={vi.fn()} {...props} />);

const row = (name: string | RegExp) => screen.getByRole("button", { name });

describe("the keys of the tree", () => {
  it("renames with F2 a plain folder, and not the one of Agents", () => {
    const onRename = vi.fn();
    const agents = folder("ag", "AGENTS", { icon: "robot" });
    draw({ folders: [casa, agents], docs: [] }, { onRename });

    fireEvent.keyDown(row("Casa"), { key: "F2" });
    expect(onRename).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(row("Agents"), { key: "F2" });

    expect(onRename).toHaveBeenCalledTimes(1);
  });

  it("asks to delete the focused document or folder with Delete", () => {
    const onDelete = vi.fn();
    draw(papers, { onDelete });

    fireEvent.keyDown(row("Dos"), { key: "Delete" });
    fireEvent.keyDown(row("Casa"), { key: "Delete" });

    expect(onDelete.mock.calls).toEqual([
      [{ id: "d2", kind: "doc" }],
      [{ id: "f1", kind: "folder" }],
    ]);
  });

  it("leaves an archived folder alone with Delete, as its menu does", () => {
    const onDelete = vi.fn();
    const old = folder("f9", "Vieja", { away: true, archived: true });
    draw({ folders: [casa, old], docs: [] }, { onDelete });
    fireEvent.click(row("Archived"));

    fireEvent.keyDown(row("Vieja"), { key: "Delete" });

    expect(onDelete).not.toHaveBeenCalled();
  });

  it("never asks to delete the shelf of loose documents", () => {
    const onDelete = vi.fn();
    draw(papers, { onDelete });

    fireEvent.keyDown(row(t("unfiled")), { key: "Delete" });

    expect(onDelete).not.toHaveBeenCalled();
  });

  it("renames the focused folder with F2, and does nothing on a document", () => {
    const onRename = vi.fn();
    draw(papers, { onRename });

    fireEvent.keyDown(row("Uno"), { key: "F2" });
    expect(onRename).not.toHaveBeenCalled();
    fireEvent.keyDown(row("Casa"), { key: "F2" });

    expect(onRename).toHaveBeenCalledWith(casa);
  });

  it("goes to the first row with Home and to the last with End", () => {
    const { container } = draw();
    const all = Array.from(container.querySelectorAll<HTMLElement>("[data-row]"));
    const middle = row("Dos");
    middle.focus();

    fireEvent.keyDown(middle, { key: "End" });
    expect(document.activeElement).toBe(all[all.length - 1]);

    fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Home" });
    expect(document.activeElement).toBe(all[0]);
    expect(all.length).toBeGreaterThan(4);
  });

  it("keeps a single stop of the tab key for the whole tree, chevrons and pages included", () => {
    const home = paper("h1", "Libro");
    const page = paper("p1", "Hoja", { pageOf: "h1" });
    const { container } = draw({
      folders: [casa, folder("f2", "Otra")],
      docs: [home, page, ...loose],
    });

    const stops = Array.from(container.querySelectorAll<HTMLElement>("button")).filter(
      (one) => one.tabIndex >= 0,
    );

    expect(stops).toHaveLength(1);
  });
});

const Picking = ({ onOpen }: { onOpen: (doc: Filed) => void }) => {
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  return (
    <Tree
      papers={papers}
      unfolded
      picked={picked}
      onPicked={(docs) => setPicked(new Set(docs))}
      onOpen={onOpen}
      onFile={vi.fn()}
    />
  );
};

describe("picking several documents", () => {
  const picked = (name: string) => screen.queryByRole("button", { name: fill("pickedIs", name) });

  it("picks and lets go of a document with Ctrl+click, without opening it", () => {
    const onOpen = vi.fn();
    render(<Picking onOpen={onOpen} />);

    fireEvent.click(row("Uno"), { ctrlKey: true });
    fireEvent.click(row("Tres"), { ctrlKey: true });
    expect([picked("Uno"), picked("Tres")].every(Boolean)).toBe(true);
    expect(picked("Dos")).toBeNull();

    fireEvent.click(picked("Uno") as HTMLElement, { ctrlKey: true });
    expect(picked("Uno")).toBeNull();
    expect(picked("Tres")).toBeTruthy();
    expect(onOpen).not.toHaveBeenCalled();
  });

  it("picks the Command+click the same way", () => {
    render(<Picking onOpen={vi.fn()} />);

    fireEvent.click(row("Dos"), { metaKey: true });

    expect(picked("Dos")).toBeTruthy();
  });

  it("picks the run from the last one picked to the one clicked with Shift, in either direction", () => {
    const onOpen = vi.fn();
    render(<Picking onOpen={onOpen} />);

    fireEvent.click(row("Cuatro"), { ctrlKey: true });
    fireEvent.click(row("Dos"), { shiftKey: true });

    expect(["Dos", "Tres", "Cuatro"].map(picked).every(Boolean)).toBe(true);
    expect(picked("Uno")).toBeNull();
    expect(onOpen).not.toHaveBeenCalled();
  });

  it("lets go of every pick with Escape", () => {
    render(<Picking onOpen={vi.fn()} />);
    fireEvent.click(row("Uno"), { ctrlKey: true });
    fireEvent.click(row("Dos"), { ctrlKey: true });

    fireEvent.keyDown(picked("Uno") as HTMLElement, { key: "Escape" });

    expect(picked("Uno")).toBeNull();
    expect(picked("Dos")).toBeNull();
  });

  it("lets go of the picks and opens the document on a plain click", () => {
    const onOpen = vi.fn();
    render(<Picking onOpen={onOpen} />);
    fireEvent.click(row("Uno"), { ctrlKey: true });

    fireEvent.click(row("Tres"));

    expect(picked("Uno")).toBeNull();
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "d3" }));
  });

  it("opens a document on Ctrl+click when nothing takes picks", () => {
    const onOpen = vi.fn();
    draw(papers, { onOpen });

    fireEvent.click(row("Uno"), { ctrlKey: true });

    expect(onOpen).toHaveBeenCalledTimes(1);
  });
});

describe("what a row can be dragged with", () => {
  const board = paper("i1", "Ideas", { file: "ideas-file" });
  const book = paper("b1", "Diario 2026", { diary: true });
  const day = paper("p1", "Lunes", { pageOf: "b1" });

  const carried = (container: HTMLElement, id: string) => {
    const one = container.querySelector<HTMLElement>(`[data-row="${id}"]`) as HTMLElement;
    fireEvent.pointerDown(one, { button: 0, clientX: 0, clientY: 0 });
    fireEvent.pointerMove(window, { clientX: 0, clientY: 40 });
    const lifted = one.className.includes("opacity-45");
    fireEvent.pointerUp(window, { clientX: 0, clientY: 40 });
    return lifted;
  };

  it("lets a plain document be dragged", () => {
    const { container } = draw();

    expect(carried(container, "d1")).toBe(true);
  });

  it("keeps the Ideas sheet and a diary book from being dragged, or taking a page dropped on them", () => {
    const { container } = draw(
      { folders: [], docs: [board, book, day, ...loose] },
      { pinned: board.file, onDiary: vi.fn(), open: day.file },
    );

    expect(carried(container, "i1")).toBe(false);
    expect(carried(container, "b1")).toBe(false);
    for (const id of ["i1", "b1"]) {
      expect(container.querySelector(`[data-drop="${id}"]`)).toBeNull();
    }
  });
});

describe("the rows of an assistant's folder", () => {
  const agents = folder("a1", "AGENTS", { icon: "robot" });
  const claude = folder("a2", "Claude", { parent: "a1", holds: 4 });
  const files = [
    paper("r1", "claude / rules/horario.md", { folder: "a2" }),
    paper("s1", "claude / pr/SKILL.md", { folder: "a2" }),
    paper("s2", "claude / pr/references/body.md", { folder: "a2" }),
    paper("m1", "claude / MEMORY.md", { folder: "a2" }),
  ];
  const found = (file: Filed, name: string, label: string, kind: AgentRow["kind"]): AgentRow => ({
    file: file.file,
    kind,
    level: "claude",
    name,
    label,
    skill: kind === "skill" ? name.split("/")[0] : null,
  });
  const all: AgentRow[] = [
    found(files[0], "rules/horario.md", "Horario", "rule"),
    found(files[1], "pr/SKILL.md", "pr", "skill"),
    found(files[2], "pr/references/body.md", "Body", "skill"),
    found(files[3], "MEMORY.md", "MEMORY.md", "memory"),
  ];

  const headings = (container: HTMLElement) =>
    Array.from(container.querySelectorAll("li"))
      .filter((one) => !one.querySelector("button") && one.textContent)
      .map((one) => one.textContent);

  const drawn = async (given: Filed[], known: AgentRow[]) => {
    rows.answer = known;
    const view = render(
      <Tree
        papers={{ folders: [agents, claude], docs: given }}
        unfolded
        onOpen={vi.fn()}
        onFile={vi.fn()}
      />,
    );
    await act(() => new Promise((soon) => setTimeout(soon, 0)));
    return view;
  };

  it("groups rules, skills and memory under their headings when the kinds are mixed", async () => {
    const { container } = await drawn(files, all);

    expect(headings(container)).toEqual([
      t("agentsKindRule"),
      t("agentsKindSkill"),
      t("agentsKindMemory"),
    ]);
  });

  it("draws no heading when everything inside is of one kind", async () => {
    const { container } = await drawn([files[0]], [all[0]]);

    expect(row("Horario")).toBeTruthy();
    expect(headings(container)).toEqual([]);
  });
});

describe("the archived header", () => {
  it("opens no menu on a right click", () => {
    const onHereMenu = vi.fn();
    const old = paper("o1", "Viejo", { archived: true, away: true });
    draw({ folders: [], docs: [old] }, { onHereMenu });

    fireEvent.contextMenu(row(t("archived")));

    expect(onHereMenu).not.toHaveBeenCalled();
  });
});
