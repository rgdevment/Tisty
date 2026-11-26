import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Papers } from "../core";
import { dropLater, fireAll, undo } from "../dropLater";
import { showAll } from "../hiding";
import { fill, t } from "../locales";
import SideLibrary, { type LibraryActs } from "../ui/SideLibrary";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "icons" ? [] : null),
}));

const paper = (id: string, title: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const docs = [paper("d1", "Uno"), paper("d2", "Dos"), paper("d3", "Tres")];
const papers: Papers = { folders: [], docs };

const acts = (): LibraryActs => ({
  newDoc: vi.fn(),
  newTitle: "New document here",
  dropDocs: vi.fn(),
  dropFolder: vi.fn(),
  rename: vi.fn(),
  moveDocs: vi.fn(),
  awayDocs: vi.fn(),
});

const draw = (given: LibraryActs | undefined, shown: Papers = papers) => {
  const hands = {
    onLean: vi.fn(),
    onChoose: vi.fn(),
    onHere: vi.fn(),
    onMove: vi.fn(),
    onFile: vi.fn(),
    onFolderMenu: vi.fn(),
    onDocMenu: vi.fn(),
    onDocsMenu: vi.fn(),
    onHereMenu: vi.fn(),
  };
  render(
    <SideLibrary
      papers={shown}
      chosen={{ named: "docs" }}
      leaning="docs"
      acts={given}
      {...hands}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: fill("openFolder", t("unfiled")) }));
  return hands;
};

const row = (name: string) => screen.getByRole("button", { name });
const pick = (...names: string[]) => {
  for (const name of names) fireEvent.click(row(name), { ctrlKey: true });
};
const bar = () => screen.queryByRole("toolbar");

beforeEach(() => {
  showAll();
});

afterEach(() => {
  fireAll();
});

describe("the New button of the library", () => {
  it("makes a document with a click, and carries the place it will be made in", () => {
    const given = acts();
    draw(given);

    const button = screen.getByRole("button", { name: t("newShort") });
    fireEvent.click(button);

    expect(given.newDoc).toHaveBeenCalledTimes(1);
    expect(button.getAttribute("title")).toBe("New document here");
  });

  it("opens the rest of the actions from its chevron, not from New", () => {
    const given = acts();
    const hands = draw(given);

    fireEvent.click(screen.getByRole("button", { name: t("docsActions") }));

    expect(hands.onDocsMenu).toHaveBeenCalledTimes(1);
    expect(given.newDoc).not.toHaveBeenCalled();
  });

  it("draws no New button when the library is given no actions", () => {
    draw(undefined);

    expect(screen.queryByRole("button", { name: t("newShort") })).toBeNull();
    expect(screen.getByRole("button", { name: t("docsActions") })).toBeTruthy();
  });
});

describe("the bar of picked documents", () => {
  it("shows nothing for no pick and for one", () => {
    draw(acts());
    expect(bar()).toBeNull();

    pick("Uno");

    expect(bar()).toBeNull();
  });

  it("appears with two picks and says how many", () => {
    draw(acts());

    pick("Uno", "Dos");

    expect(screen.getByRole("toolbar", { name: fill("pickedMany", "2") })).toBeTruthy();
  });

  it("deletes the picked documents, and only them, and lets go of the pick", () => {
    const given = acts();
    draw(given);
    pick("Uno", "Tres");

    fireEvent.click(within(bar() as HTMLElement).getByRole("button", { name: t("deleteIt") }));

    expect(vi.mocked(given.dropDocs).mock.calls[0][0].map((one) => one.id)).toEqual(["d1", "d3"]);
    expect(bar()).toBeNull();
  });

  it("archives the picked documents and lets go of the pick", () => {
    const given = acts();
    draw(given);
    pick("Dos", "Tres");

    fireEvent.click(within(bar() as HTMLElement).getByRole("button", { name: t("putAway") }));

    expect(vi.mocked(given.awayDocs).mock.calls[0][0].map((one) => one.id)).toEqual(["d2", "d3"]);
    expect(bar()).toBeNull();
  });

  it("asks where to move the picked documents from the place of its button, and keeps the pick", () => {
    const given = acts();
    draw(given);
    pick("Uno", "Dos");

    fireEvent.click(within(bar() as HTMLElement).getByRole("button", { name: t("moveTo") }));

    const [moved, at] = vi.mocked(given.moveDocs).mock.calls[0];
    expect(moved.map((one) => one.id)).toEqual(["d1", "d2"]);
    expect(at).toEqual({ x: expect.any(Number), y: expect.any(Number) });
    expect(bar()).toBeTruthy();
  });

  it("lets go of the pick from its own button", () => {
    draw(acts());
    pick("Uno", "Dos");

    fireEvent.click(screen.getByRole("button", { name: t("pickedLetGo") }));

    expect(bar()).toBeNull();
    expect(screen.queryByRole("button", { name: fill("pickedIs", "Uno") })).toBeNull();
  });

  it("deletes the whole pick with the Delete key on one of them", () => {
    const given = acts();
    draw(given);
    pick("Uno", "Dos");

    fireEvent.keyDown(screen.getByRole("button", { name: fill("pickedIs", "Dos") }), {
      key: "Delete",
    });

    expect(vi.mocked(given.dropDocs).mock.calls[0][0].map((one) => one.id)).toEqual(["d1", "d2"]);
  });

  it("deletes only the focused document with Delete when it is not one of the pick", () => {
    const given = acts();
    draw(given);
    pick("Uno", "Dos");

    fireEvent.keyDown(row("Tres"), { key: "Delete" });

    expect(vi.mocked(given.dropDocs).mock.calls[0][0].map((one) => one.id)).toEqual(["d3"]);
  });

  it("asks to delete a folder with Delete, not its documents", () => {
    const given = acts();
    const casa = {
      id: "f1",
      name: "Casa",
      parent: null,
      icon: null,
      holds: 0,
      archived: false,
      away: false,
    };
    draw(given, { folders: [casa], docs });

    fireEvent.keyDown(row("Casa"), { key: "Delete" });

    expect(given.dropFolder).toHaveBeenCalledWith(casa);
    expect(given.dropDocs).not.toHaveBeenCalled();
  });

  it("renames a folder with F2 through the actions", () => {
    const given = acts();
    const casa = {
      id: "f1",
      name: "Casa",
      parent: null,
      icon: null,
      holds: 0,
      archived: false,
      away: false,
    };
    draw(given, { folders: [casa], docs });

    fireEvent.keyDown(row("Casa"), { key: "F2" });

    expect(given.rename).toHaveBeenCalledWith(casa);
  });
});

describe("what waits to be deleted", () => {
  it("is not drawn, and is drawn again when the delete is undone", () => {
    draw(acts());
    expect(row("Dos")).toBeTruthy();

    let key = "";
    act(() => {
      key = dropLater({ hides: ["d2"], said: "gone", run: () => Promise.resolve() });
    });

    expect(screen.queryByRole("button", { name: "Dos" })).toBeNull();
    expect(row("Uno")).toBeTruthy();

    act(() => undo(key));

    expect(row("Dos")).toBeTruthy();
  });

  it("leaves a pending document out of the pick too", () => {
    draw(acts());
    pick("Uno", "Dos");
    expect(bar()).toBeTruthy();

    act(() => {
      dropLater({ hides: ["d2"], said: "gone", run: () => Promise.resolve() });
    });

    expect(bar()).toBeNull();
  });
});
