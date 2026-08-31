import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Filed, Folded } from "../core";
import Folder from "../ui/Folder";

const folders: Folded[] = [
  { id: "1", name: "trabajo", parent: null, icon: "briefcase", color: "blue", holds: 2 },
  { id: "2", name: "corporativo", parent: "1", icon: null, color: null, holds: 1 },
  { id: "3", name: "actas", parent: "2", icon: null, color: null, holds: 0 },
  { id: "4", name: "personal", parent: null, icon: null, color: null, holds: 0 },
];

const docs: Filed[] = [
  { id: "a", file: "0001", title: "Contrato", folder: "2", archived: false },
  { id: "b", file: "0002", title: "", folder: "2", archived: false },
  { id: "c", file: "0003", title: "Viejo", folder: "2", archived: true },
  { id: "d", file: "0004", title: "Ajeno", folder: "4", archived: false },
];

const show = (which: string) => {
  const onOpen = vi.fn();
  const onHere = vi.fn();
  const onMenu = vi.fn();
  const onDocMenu = vi.fn();
  const folder = folders.find((one) => one.id === which) as Folded;
  const view = render(
    <Folder
      folder={folder}
      folders={folders}
      docs={docs}
      onOpen={onOpen}
      onHere={onHere}
      onMenu={onMenu}
      onDocMenu={onDocMenu}
    />,
  );
  return { ...view, onOpen, onHere, onMenu, onDocMenu };
};

describe("the folder view", () => {
  it("lists the folders and the papers it holds, and nobody else's", () => {
    show("2");

    expect(screen.getByRole("button", { name: /^actas/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Contrato/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Ajeno/ })).toBeNull();
  });

  it("leaves an archived paper where it was put away", () => {
    show("2");

    expect(screen.queryByRole("button", { name: /Viejo/ })).toBeNull();
  });

  it("counts what is inside, one at a time or many", () => {
    show("2");

    expect(screen.getByText(/1 folder · 2 papers/)).toBeTruthy();
  });

  it("counts a lone paper without pluralising it", () => {
    show("4");

    expect(screen.getByText(/^1 paper$/)).toBeTruthy();
  });

  it("says so plainly when there is nothing inside", () => {
    show("3");

    expect(screen.getByText("nothing in here yet")).toBeTruthy();
  });

  it("walks the way back up, deepest last", () => {
    show("3");

    const trail = screen.getByRole("navigation", { name: "Where you are" });
    const steps = Array.from(trail.querySelectorAll("button")).map((one) => one.textContent);
    expect(steps).toEqual(["trabajo", "corporativo"]);
  });

  it("shows no trail at the top of the tree", () => {
    show("1");

    expect(screen.queryByRole("navigation", { name: "Where you are" })).toBeNull();
  });

  it("walks into a folder you click and opens a paper you click", async () => {
    const { onHere, onOpen } = show("2");

    await userEvent.click(screen.getByRole("button", { name: /^actas/ }));
    expect(onHere).toHaveBeenCalledWith("3");

    await userEvent.click(screen.getByRole("button", { name: /Contrato/ }));
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ file: "0001" }));
  });

  it("climbs back through the trail", async () => {
    const { onHere } = show("3");

    await userEvent.click(screen.getByRole("button", { name: "trabajo" }));
    expect(onHere).toHaveBeenCalledWith("1");
  });

  it("names a paper that never got a title", () => {
    show("2");

    expect(screen.getByRole("button", { name: /Untitled/ })).toBeTruthy();
  });

  it("hands the right thing to the menu that opens over it", () => {
    const { onMenu, onDocMenu } = show("2");

    fireRight(screen.getByRole("button", { name: /^actas/ }));
    expect(onMenu).toHaveBeenCalledWith(
      expect.objectContaining({ id: "3" }),
      expect.objectContaining({ x: expect.any(Number) }),
    );

    fireRight(screen.getByRole("button", { name: /Contrato/ }));
    expect(onDocMenu).toHaveBeenCalledWith(
      expect.objectContaining({ file: "0001" }),
      expect.objectContaining({ x: expect.any(Number) }),
    );
  });
});

const fireRight = (at: HTMLElement) => {
  at.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, clientX: 10, clientY: 20 }));
};

describe("reaching the folder view without a mouse", () => {
  it("opens a folder's menu from the keyboard, where a Mac has no context key", async () => {
    const { onMenu } = show("2");
    screen.getByRole("button", { name: /^actas/ }).focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onMenu).toHaveBeenCalledWith(
      expect.objectContaining({ id: "3" }),
      expect.objectContaining({ x: expect.any(Number), y: expect.any(Number) }),
    );
  });

  it("opens a paper's menu from the keyboard too", async () => {
    const { onDocMenu } = show("2");
    screen.getByRole("button", { name: /Contrato/ }).focus();

    await userEvent.keyboard("{ContextMenu}");

    expect(onDocMenu).toHaveBeenCalledWith(
      expect.objectContaining({ file: "0001" }),
      expect.any(Object),
    );
  });

  it("opens the menu of the folder you are standing in, which the sidebar alone used to hold", async () => {
    const { onMenu } = show("2");
    const title = screen.getByRole("button", { name: /corporativo/i });
    title.focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onMenu).toHaveBeenCalledWith(expect.objectContaining({ id: "2" }), expect.any(Object));
  });

  it("says nothing to a key that is not asking for a menu", async () => {
    const { onMenu } = show("2");
    screen.getByRole("button", { name: /^actas/ }).focus();

    await userEvent.keyboard("{F10}a");

    expect(onMenu).not.toHaveBeenCalled();
  });
});

describe("the loose papers, which belong to no folder", () => {
  const loose = () => {
    const onOpen = vi.fn();
    const onHereMenu = vi.fn();
    render(
      <Folder
        folder={null}
        folders={folders}
        docs={[...docs, { id: "e", file: "0005", title: "Suelto", folder: null, archived: false }]}
        onOpen={onOpen}
        onHere={vi.fn()}
        onHereMenu={onHereMenu}
      />,
    );
    return { onOpen, onHereMenu };
  };

  it("gathers what no folder holds, including a paper whose folder is gone", () => {
    loose();

    expect(screen.getByRole("button", { name: /Suelto/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Contrato/ })).toBeNull();
  });

  it("has no trail and no folders of its own", () => {
    loose();

    expect(screen.queryByRole("navigation", { name: "Where you are" })).toBeNull();
    expect(screen.queryByText("Folders")).toBeNull();
  });

  it("opens the menu that belongs to nowhere in particular", async () => {
    const { onHereMenu } = loose();
    screen.getByRole("button", { name: /Unfiled/ }).focus();

    await userEvent.keyboard("{Shift>}{F10}{/Shift}");

    expect(onHereMenu).toHaveBeenCalledWith(expect.any(Object));
  });

  it("leaves the title alone when nobody is listening for a menu", () => {
    render(
      <Folder folder={null} folders={folders} docs={docs} onOpen={vi.fn()} onHere={vi.fn()} />,
    );

    expect(screen.getByRole("button", { name: /Unfiled/ })).toHaveProperty("disabled", true);
  });
});

describe("what a folder shows without being entered", () => {
  const deep: Folded[] = [
    { id: "1", name: "personal", parent: null, icon: null, color: null, holds: 0 },
    { id: "2", name: "condominio", parent: "1", icon: null, color: null, holds: 12 },
    { id: "3", name: "moto", parent: "1", icon: null, color: null, holds: 1 },
  ];
  const many: Filed[] = Array.from({ length: 12 }, (_, i) => ({
    id: `c${i}`,
    file: `10${i}`,
    title: `Acta ${i}`,
    folder: "2",
    archived: false,
  })).concat([{ id: "m", file: "200", title: "Revisión", folder: "3", archived: false }]);

  const open = () => {
    const onHere = vi.fn();
    render(
      <Folder
        folder={deep[0]}
        folders={deep}
        docs={many}
        onOpen={vi.fn()}
        onHere={onHere}
        onMenu={vi.fn()}
      />,
    );
    return { onHere };
  };

  it("counts what lies below, and what is right here", () => {
    open();

    expect(screen.getByText("2 folders · 13 papers in all")).toBeTruthy();
  });

  it("shows what each folder holds without going into it", () => {
    open();

    expect(screen.getByRole("button", { name: /^Acta 0/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /^Revisión/ })).toBeTruthy();
  });

  it("stops at a handful and offers the rest where they live", async () => {
    const { onHere } = open();

    expect(screen.queryByRole("button", { name: /^Acta 11/ })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: /4 more/ }));

    expect(onHere).toHaveBeenCalledWith("2");
  });

  it("folds one away and leaves the other where it was", async () => {
    open();

    await userEvent.click(screen.getByRole("button", { name: "Close condominio" }));

    expect(screen.queryByRole("button", { name: /^Acta 0/ })).toBeNull();
    expect(screen.getByRole("button", { name: /^Revisión/ })).toBeTruthy();
  });
});
