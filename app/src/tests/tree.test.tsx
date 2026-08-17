import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Papers } from "../core";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve([["work", "💼"]]),
}));

const papers: Papers = {
  folders: [
    { id: "01F", name: "trabajo", parent: null, icon: "work", holds: 3 },
    { id: "01G", name: "corporativo", parent: "01F", icon: null, holds: 1 },
    { id: "01H", name: "personal", parent: null, icon: null, holds: 0 },
  ],
  docs: [
    { id: "01A", file: "a3f1-0001", title: "Compras", folder: "01F", archived: false },
    { id: "01B", file: "a3f1-0002", title: "Contrato", folder: "01G", archived: false },
    { id: "01C", file: "a3f1-0003", title: "Suelto", folder: null, archived: false },
    { id: "01D", file: "a3f1-0004", title: "", folder: null, archived: false },
    { id: "01E", file: "a3f1-0005", title: "Viejo", folder: "01F", archived: true },
    { id: "01J", file: "a3f1-0006", title: "", folder: null, archived: false, gone: true },
  ],
};

describe("the document tree", () => {
  const show = (onFile = vi.fn(), onOpen = vi.fn(), here?: string | null) => {
    const onFolderMenu = vi.fn();
    const onDocMenu = vi.fn();
    const onHere = vi.fn();
    const onMove = vi.fn();
    render(
      <Tree
        papers={papers}
        here={here}
        onOpen={onOpen}
        onFile={onFile}
        onHere={onHere}
        onMove={onMove}
        onFolderMenu={onFolderMenu}
        onDocMenu={onDocMenu}
      />,
    );
    return { onFile, onOpen, onFolderMenu, onDocMenu, onHere, onMove };
  };

  it("hangs each folder from the one it belongs to", () => {
    show();

    const work = screen.getByRole("button", { name: "Close trabajo" });
    const inside = screen.getByRole("button", { name: "Close corporativo" });
    expect(inside.style.marginLeft).not.toBe(work.style.marginLeft);
  });

  it("starts a document to the right of the folder holding it", () => {
    show();
    const work = screen.getByRole("button", { name: "Close trabajo" });
    const inside = screen.getByRole("button", { name: "Compras" });

    const folderText = parseInt(work.style.marginLeft, 10) + 12 + 6 + 21;
    const docText = parseInt(inside.style.paddingLeft, 10) + 12 + 6;

    expect(docText).toBeGreaterThan(folderText);
  });

  it("counts what hangs below a folder, not only what it holds", () => {
    show();

    expect(screen.getByRole("button", { name: "trabajo" }).textContent).toContain("3");
  });

  it("says whether a branch is open, for a reader that cannot see it", async () => {
    show();
    expect(
      screen.getByRole("button", { name: "Close trabajo" }).getAttribute("aria-expanded"),
    ).toBe("true");

    await userEvent.click(screen.getByRole("button", { name: "Close trabajo" }));

    expect(screen.getByRole("button", { name: "Open trabajo" }).getAttribute("aria-expanded")).toBe(
      "false",
    );
  });

  it("picks a folder and folds it in the same click", async () => {
    const { onHere } = show();

    await userEvent.click(screen.getByRole("button", { name: "trabajo" }));

    expect(onHere).toHaveBeenCalledWith("01F");
    expect(screen.queryByRole("button", { name: "corporativo" })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: "trabajo" }));

    expect(screen.getByRole("button", { name: "corporativo" })).toBeTruthy();
  });

  it("folds unfiled away too, and says so", async () => {
    const { onHere } = show();
    const loose = screen.getByRole("button", { name: /Unfiled/ });
    expect(loose.getAttribute("aria-expanded")).toBe("true");

    await userEvent.click(loose);

    expect(onHere).toHaveBeenCalledWith(undefined);
    expect(screen.queryByRole("button", { name: "Suelto" })).toBeNull();
    expect(screen.getByRole("button", { name: /Unfiled/ }).getAttribute("aria-expanded")).toBe(
      "false",
    );
  });

  it("lets unfiled be picked like any other place", async () => {
    const { onHere } = show();

    await userEvent.click(screen.getByRole("button", { name: /Unfiled/ }));

    expect(onHere).toHaveBeenCalledWith(undefined);
  });

  it("marks the place being looked at", () => {
    show(vi.fn(), vi.fn(), null);

    expect(screen.getByRole("button", { name: /Unfiled/ }).getAttribute("aria-current")).toBe(
      "true",
    );
  });

  it("hangs a folder from the one it was dropped on", () => {
    const { onMove } = show();
    const work = screen.getByRole("button", { name: "trabajo" }).parentElement
      ?.parentElement as HTMLElement;

    fireEvent.drop(work, {
      dataTransfer: { getData: (kind: string) => (kind === "text/tisty-folder" ? "01H" : "") },
    });

    expect(onMove).toHaveBeenCalledWith("01H", "01F");
  });

  it("never hangs a folder from itself", () => {
    const { onMove } = show();
    const work = screen.getByRole("button", { name: "trabajo" }).parentElement
      ?.parentElement as HTMLElement;

    fireEvent.drop(work, {
      dataTransfer: { getData: (kind: string) => (kind === "text/tisty-folder" ? "01F" : "") },
    });

    expect(onMove).not.toHaveBeenCalled();
  });

  it("always offers somewhere for what is not filed", () => {
    show();

    expect(screen.getByText("Unfiled")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Suelto" })).toBeTruthy();
  });

  it("names a document that has nothing written in it yet", () => {
    show();

    const row = document.querySelector('[data-row="01D"]');

    expect(row?.getAttribute("aria-label")).toBe("Untitled");
    expect(row?.textContent).not.toMatch(/⚠/);
  });

  it("folds a branch away and brings it back", async () => {
    show();
    expect(screen.getByRole("button", { name: "corporativo" })).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: "Close trabajo" }));

    expect(screen.queryByRole("button", { name: "corporativo" })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: "Open trabajo" }));

    expect(screen.getByRole("button", { name: "corporativo" })).toBeTruthy();
  });

  it("opens a document by the file a reference points at", async () => {
    const { onOpen } = show();

    await userEvent.click(screen.getByRole("button", { name: "Compras" }));

    expect(onOpen.mock.calls[0][0].file).toBe("a3f1-0001");
  });

  it("opens the folder menu from the keyboard, with no mouse anywhere", async () => {
    const { onFolderMenu } = show();
    screen.getByRole("button", { name: "corporativo" }).focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onFolderMenu.mock.calls[0][0].id).toBe("01G");
  });

  it("opens a menu on right click without the browser one", () => {
    const { onFolderMenu } = show();
    const row = screen.getByRole("button", { name: "trabajo" }).parentElement as HTMLElement;

    fireEvent.contextMenu(row, { clientX: 40, clientY: 90 });

    expect(onFolderMenu.mock.calls[0][0].id).toBe("01F");
    expect(onFolderMenu.mock.calls[0][1]).toEqual({ x: 40, y: 90 });
  });

  it("opens the document menu from the keyboard too", async () => {
    const { onDocMenu } = show();
    screen.getByRole("button", { name: "Compras" }).focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onDocMenu.mock.calls[0][0].id).toBe("01A");
  });

  it("opens a document menu on right click", () => {
    const { onDocMenu } = show();
    const row = screen.getByRole("button", { name: "Compras" }).parentElement as HTMLElement;

    fireEvent.contextMenu(row, { clientX: 12, clientY: 34 });

    expect(onDocMenu.mock.calls[0][0].id).toBe("01A");
  });

  it("moves a document into a folder with no mouse at all", async () => {
    const { onFile } = show();

    screen.getByRole("button", { name: "Compras" }).focus();
    await userEvent.keyboard("{Control>}x{/Control}");
    expect(screen.getByRole("status").textContent).toContain("Compras");

    screen.getByRole("button", { name: "personal" }).focus();
    await userEvent.keyboard("{Control>}v{/Control}");

    expect(onFile).toHaveBeenCalledWith("01A", "01H");
  });

  it("takes a document out of every folder onto unfiled with the keyboard", async () => {
    const { onFile } = show();

    screen.getByRole("button", { name: "Compras" }).focus();
    await userEvent.keyboard("{Control>}x{/Control}");
    screen.getByRole("button", { name: /Unfiled/ }).focus();
    await userEvent.keyboard("{Control>}v{/Control}");

    expect(onFile).toHaveBeenCalledWith("01A", undefined);
  });

  it("nests a folder with the keyboard, and never inside itself", async () => {
    const { onMove } = show();

    screen.getByRole("button", { name: "personal" }).focus();
    await userEvent.keyboard("{Control>}x{/Control}");

    const lifted = screen.getByRole("button", { name: "personal, lifted" });
    lifted.focus();
    await userEvent.keyboard("{Control>}v{/Control}");

    expect(onMove).not.toHaveBeenCalled();

    screen.getByRole("button", { name: "trabajo" }).focus();
    await userEvent.keyboard("{Control>}v{/Control}");

    expect(onMove).toHaveBeenCalledWith("01H", "01F");
  });

  it("lets go of what it lifted when told to", async () => {
    const { onFile } = show();

    screen.getByRole("button", { name: "Compras" }).focus();
    await userEvent.keyboard("{Control>}x{/Control}");
    await userEvent.keyboard("{Escape}");

    expect(screen.queryByRole("status")).toBeNull();

    screen.getByRole("button", { name: "personal" }).focus();
    await userEvent.keyboard("{Control>}v{/Control}");

    expect(onFile).not.toHaveBeenCalled();
  });

  it("walks the tree with the arrow keys", async () => {
    show();
    const work = screen.getByRole("button", { name: "trabajo" });
    work.focus();

    await userEvent.keyboard("{ArrowDown}");

    expect(document.activeElement).not.toBe(work);
    expect((document.activeElement as HTMLElement).dataset.row).toBeTruthy();
  });

  it("opens and closes a branch with the side arrows", async () => {
    show();
    screen.getByRole("button", { name: "trabajo" }).focus();

    await userEvent.keyboard("{ArrowLeft}");

    expect(screen.queryByRole("button", { name: "corporativo" })).toBeNull();

    screen.getByRole("button", { name: "trabajo" }).focus();
    await userEvent.keyboard("{ArrowRight}");

    expect(screen.getByRole("button", { name: "corporativo" })).toBeTruthy();
  });

  it("leaves no stray buttons in the rows", () => {
    show();

    expect(screen.queryByRole("button", { name: /More options/ })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Compras" }).getAttribute("aria-keyshortcuts"),
    ).toContain("Shift+F10");
  });

  it("takes a document dropped anywhere on the unfiled list", () => {
    const { onFile } = show();
    const list = screen.getByRole("button", { name: "Suelto" }).closest("ul") as HTMLElement;

    fireEvent.drop(list, { dataTransfer: { getData: () => "01A" } });

    expect(onFile).toHaveBeenCalledWith("01A", undefined);
  });

  it("keeps what was archived out of its folder and in its own place", () => {
    show();

    expect(screen.getByRole("button", { name: "trabajo" }).textContent).toContain("3");
    expect(screen.getByRole("button", { name: "Viejo" })).toBeTruthy();
    expect(
      screen
        .getByRole("list", { name: "Documents" })
        .contains(screen.getByRole("button", { name: "Viejo" })),
    ).toBe(false);
  });

  it("folds the archived away, since that is the point of it", async () => {
    show();

    await userEvent.click(screen.getByRole("button", { name: "Archived" }));

    expect(screen.queryByRole("button", { name: "Viejo" })).toBeNull();
  });

  it("offers making something on the shelf itself, not only on a folder", () => {
    const onHereMenu = vi.fn();
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHereMenu={onHereMenu} />);

    fireEvent.contextMenu(screen.getByRole("button", { name: /Unfiled/ }), {
      clientX: 5,
      clientY: 9,
    });

    expect(onHereMenu).toHaveBeenCalledWith({ x: 5, y: 9 });
  });

  it("never treats the unfiled shelf as a folder that can be renamed or deleted", async () => {
    const { onFolderMenu } = show();
    screen.getByRole("button", { name: /Unfiled/ }).focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onFolderMenu).not.toHaveBeenCalled();
  });

  it("files a document into the folder it was dropped on", () => {
    const { onFile } = show();
    const work = screen.getByRole("button", { name: "trabajo" }).parentElement
      ?.parentElement as HTMLElement;

    fireEvent.drop(work, { dataTransfer: { getData: () => "01C" } });

    expect(onFile).toHaveBeenCalledWith("01C", "01F");
  });

  it("takes a document out of every folder when dropped on unfiled", () => {
    const { onFile } = show();
    const loose = screen.getByRole("button", { name: /Unfiled/ }).parentElement as HTMLElement;

    fireEvent.drop(loose, { dataTransfer: { getData: () => "01A" } });

    expect(onFile).toHaveBeenCalledWith("01A", undefined);
  });

  it("still shows a document whose file is not here, so it can be seen and removed", () => {
    show();

    const rows = screen.getAllByRole("button", { name: /untitled/i });

    expect(rows.length).toBe(2);
    expect(rows.some((one) => one.textContent?.includes("⚠"))).toBe(true);
  });

  it("marks only the one whose file is missing", () => {
    show();

    const marked = screen
      .getAllByRole("button", { name: /untitled/i })
      .filter((one) => one.textContent?.includes("⚠"));

    expect(marked).toHaveLength(1);
    expect(marked[0].querySelector("[title]")?.getAttribute("title")).toMatch(
      /not on this machine/i,
    );
  });
});
