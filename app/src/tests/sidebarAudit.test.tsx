import { fireEvent, render, screen } from "@testing-library/react";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Folded, Papers } from "../core";
import { docChoices, folderChoices, type Hands, hereChoices } from "../docMenus";
import { showAll } from "../hiding";
import { adopt, fill, locale, t } from "../locales";
import { placesRules } from "../places";
import type { Choice } from "../ui/Menu";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    Promise.resolve(
      cmd === "places_rules"
        ? {
            plain: { makes: [], pages: true, moves: true, copies: true },
            agents: { makes: [], pages: true, moves: true, copies: true },
            ideas: { makes: [], pages: false, moves: false, copies: false },
            diary: { makes: [], pages: false, moves: false, copies: true },
          }
        : [["work", "x"]],
    ),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
}));

const folder = (extra: Partial<Folded>): Folded =>
  ({
    name: "f",
    parent: null,
    icon: null,
    holds: 0,
    archived: false,
    away: false,
    ...extra,
  }) as Folded;

const paper = (extra: Partial<Filed>): Filed =>
  ({ title: "t", folder: null, archived: false, away: false, locked: false, ...extra }) as Filed;

const casa = folder({ id: "f1", name: "Casa" });
const agentsRoot = folder({ id: "a1", name: "AGENTS", icon: "robot" });
const agentsClaude = folder({ id: "a2", name: "Claude", parent: "a1" });

const doc = paper({ id: "d1", file: "dev_a-0001", title: "Minuta", folder: "f1" });
const sibling = paper({ id: "d2", file: "dev_a-0002", title: "Otra", folder: "f1" });
const book = paper({ id: "b1", file: "dev_a-0003", title: "Diario 2026", diary: true });

const handsFor = (papers: Papers, showing: string | null = null) => {
  const h = {
    papers,
    showing,
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

const shown = (choices: Choice[]) => choices.filter((one) => !one.off);
const keysOf = (choices: Choice[]) => shown(choices).map((one) => one.key);

beforeAll(async () => {
  await placesRules();
});

beforeEach(() => {
  showAll();
});

describe("the menus of the documents sidebar", () => {
  it("gives a diary book only the choices a diary book takes", () => {
    const h = handsFor({ folders: [], docs: [book] });

    const offered = keysOf(docChoices(book, h));

    for (const key of ["newPage", "pageOf", "move", "translate", "copy", "lock"]) {
      expect(offered).not.toContain(key);
    }
  });

  it("keeps diagrams, folders, import and unpack out of the plus menu inside Agents", () => {
    const h = handsFor({ folders: [agentsRoot, agentsClaude], docs: [] });

    const offered = keysOf(hereChoices(h, "a2"));

    for (const key of ["newDiagram", "newFolder", "import", "unpack"]) {
      expect(offered).not.toContain(key);
    }
  });

  it("creates a folder at the root when the root menu picks New folder (fix through makeFolderIn)", () => {
    const h = handsFor({ folders: [casa], docs: [] });

    hereChoices(h)
      .find((one) => one.key === "newFolder")
      ?.onPick?.();

    expect(h.makeFolderIn).toHaveBeenCalledWith(undefined);
  });

  it("keeps every folder and document menu to at most ten visible choices", () => {
    const papers = { folders: [casa], docs: [doc, sibling] } as Papers;
    const open = handsFor(papers, doc.file);
    const closed = handsFor(papers, null);

    const counts = {
      folder: shown(folderChoices(casa, open)).length,
      openDoc: shown(docChoices(doc, open)).length,
      closedDoc: shown(docChoices(doc, closed)).length,
    };

    expect(Math.max(counts.folder, counts.openDoc, counts.closedDoc)).toBeLessThanOrEqual(10);
  });
});

describe("the tree of the documents sidebar", () => {
  const draw = (papers: Papers, props: Partial<Parameters<typeof Tree>[0]> = {}) =>
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} {...props} />);

  it("keeps a document dropped on Ideas or on a diary book from becoming a page", () => {
    const board = paper({ id: "i1", file: "ideas-1", title: "Ideas" });
    const { container } = draw(
      { folders: [], docs: [board, book] },
      { pinned: board.file, open: book.file, onDiary: vi.fn(), onPage: vi.fn() },
    );

    expect(container.querySelector('[data-drop="i1"]')).toBeNull();
    expect(container.querySelector('[data-drop="b1"]')).toBeNull();
  });

  it("keeps every folder chevron out of the tab order", () => {
    const folders = [
      folder({ id: "f1", name: "uno" }),
      folder({ id: "f2", name: "dos" }),
      folder({ id: "f3", name: "tres" }),
    ];
    draw({ folders, docs: [] });

    const list = screen.getByRole("list", { name: t("docs") });
    const stops = Array.from(list.querySelectorAll<HTMLElement>("button")).filter(
      (one) => one.tabIndex >= 0,
    );

    expect(stops).toHaveLength(1);
  });

  it("folds a document whose page is the open one", () => {
    const home = paper({ id: "d1", file: "dev_a-0001", title: "Libro" });
    const page = paper({ id: "p1", file: "dev_a-0009", title: "Hoja", pageOf: "d1" });
    const { container } = draw({ folders: [], docs: [home, page] }, { open: page.file });
    fireEvent.click(screen.getByRole("button", { name: fill("openFolder", t("unfiled")) }));
    expect(container.querySelector("#pages-d1")).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: fill("hidePages", "Libro") }));

    expect(container.querySelector("#pages-d1")).toBeNull();
  });

  it("folds the diary group while one of its books is open", () => {
    const { container } = draw(
      { folders: [], docs: [book] },
      { open: book.file, onDiary: vi.fn() },
    );
    expect(container.querySelector("#pages-diary")).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: fill("hidePages", t("diary")) }));

    expect(container.querySelector("#pages-diary")).toBeNull();
  });

  it("opens the New document menu from the keyboard on the unfiled row", () => {
    const onHereMenu = vi.fn();
    draw({ folders: [], docs: [] }, { onHereMenu });

    fireEvent.keyDown(screen.getByRole("button", { name: t("unfiled") }), {
      key: "F10",
      shiftKey: true,
    });

    expect(onHereMenu).toHaveBeenCalled();
  });

  it("does not offer New document from a right click on the Archived header", () => {
    const onHereMenu = vi.fn();
    const old = paper({ id: "d1", file: "dev_a-0001", archived: true, away: true });
    draw({ folders: [], docs: [old] }, { onHereMenu });

    fireEvent.contextMenu(screen.getByRole("button", { name: t("archived") }));

    expect(onHereMenu).not.toHaveBeenCalled();
  });
});

describe("the texts of the documents sidebar", () => {
  const was = locale();

  it("names the Spanish row for loose documents in the delete confirmation", () => {
    adopt("es");
    try {
      expect(t("dropFolderSure")).toContain(t("unfiled"));
    } finally {
      adopt(was);
    }
  });
});

describe("what the documents sidebar still lacks", () => {
  it.todo("keeps deleted documents in a trash");
});
