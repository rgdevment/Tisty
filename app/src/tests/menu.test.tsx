import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Menu, { type Choice } from "../ui/Menu";

const choices = (onPick = vi.fn()): Choice[] => [
  { key: "new", label: "Nuevo documento", icon: "+", onPick },
  { key: "folder", label: "Nueva carpeta", icon: "+", off: true, onPick },
  { key: "rename", label: "Renombrar", apart: true, onPick },
  { key: "drop", label: "Borrar", danger: true, apart: true, onPick },
];

// A menu that closes itself, and whose choice may put the focus in a field of its own.
const Shown = ({ typed, field }: { typed: boolean; field: HTMLInputElement }) => {
  const [open, setOpen] = useState(true);
  const pick = vi.fn(() => {
    if (typed) field.focus();
  });
  return open ? (
    <Menu
      at={{ x: 20, y: 30 }}
      choices={choices(pick)}
      label="Opciones"
      onClose={() => setOpen(false)}
    />
  ) : null;
};

describe("where the menu hangs", () => {
  it("mounts on the body, out of reach of any ancestor that clips or contains", () => {
    const { container } = render(
      <div style={{ containerType: "inline-size", overflow: "hidden" }}>
        <Menu at={{ x: 20, y: 30 }} choices={choices()} label="Opciones" onClose={vi.fn()} />
      </div>,
    );

    const menu = screen.getByRole("menu");

    expect(menu.parentElement).toBe(document.body);
    expect(container.contains(menu)).toBe(false);
  });
});

describe("where the menu opens", () => {
  const TALL = 300;
  const WIDE = 210;
  let bar: HTMLElement | null = null;

  beforeEach(() => {
    Object.defineProperty(window, "innerHeight", { configurable: true, value: 800 });
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 1200 });
    Object.defineProperty(HTMLElement.prototype, "scrollHeight", {
      configurable: true,
      get() {
        return (this as HTMLElement).getAttribute("role") === "menu" ? TALL : 0;
      },
    });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
      this: HTMLElement,
    ) {
      if (this === bar) return { top: 45, bottom: 91, left: 0, right: 1200 } as DOMRect;
      return { top: 0, bottom: 0, left: 0, right: WIDE, width: WIDE, height: TALL } as DOMRect;
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    Reflect.deleteProperty(HTMLElement.prototype, "scrollHeight");
    bar?.remove();
    bar = null;
  });

  const placed = (at: { x: number; y: number; top?: number; right?: number }) => {
    const view = render(<Menu at={at} choices={choices()} label="Opciones" onClose={vi.fn()} />);
    const menu = screen.getByRole("menu");
    const where = {
      x: Number.parseFloat(menu.style.left),
      y: Number.parseFloat(menu.style.top),
      bottom: Number.parseFloat(menu.style.bottom),
      tall: Number.parseFloat(menu.style.maxHeight),
    };
    view.unmount();
    return where;
  };

  it("goes up over its anchor near the bottom, leaving the anchor in sight", () => {
    expect(placed({ x: 100, y: 780, top: 760 })).toMatchObject({
      bottom: 800 - (760 - 4),
      tall: TALL,
    });
  });

  it("stays below and scrolls when there is fair room there, rather than climbing the window", () => {
    expect(placed({ x: 100, y: 540, top: 520 })).toMatchObject({ y: 540, tall: 800 - 6 - 540 });
  });

  it("never climbs over the bar above the page", () => {
    bar = document.createElement("div");
    bar.className = "ribbon";
    document.body.append(bar);
    const menu = placed({ x: 100, y: 780, top: 300 });
    expect(menu.tall).toBe(300 - 4 - 91);
    expect(800 - menu.bottom - menu.tall).toBe(91);
  });

  it("hangs from its anchor's right edge in the right half of the window", () => {
    expect(placed({ x: 900, y: 100, top: 80, right: 920 }).x).toBe(920 - WIDE);
    expect(placed({ x: 100, y: 100, top: 80, right: 120 }).x).toBe(100);
  });
});

describe("the press that opened it", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("never reaches the watch for a press outside, only the one after does", () => {
    const onClose = vi.fn();
    render(<Menu at={{ x: 20, y: 30 }} choices={choices()} label="Opciones" onClose={onClose} />);

    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(onClose).not.toHaveBeenCalled();

    act(() => {
      vi.runAllTimers();
    });
    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(onClose).toHaveBeenCalled();
  });

  it("closes when a page's frame takes the focus or the page scrolls, but not for its own scroll", () => {
    const onClose = vi.fn();
    render(<Menu at={{ x: 20, y: 30 }} choices={choices()} label="Opciones" onClose={onClose} />);
    act(() => {
      vi.runAllTimers();
    });

    screen.getByRole("menu").dispatchEvent(new Event("scroll"));
    expect(onClose).not.toHaveBeenCalled();
    document.body.dispatchEvent(new Event("scroll"));
    expect(onClose).toHaveBeenCalledTimes(1);
    window.dispatchEvent(new Event("blur"));
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});

describe("the row menu", () => {
  const show = (onClose = vi.fn(), onPick = vi.fn()) => {
    render(
      <Menu at={{ x: 20, y: 30 }} choices={choices(onPick)} label="Opciones" onClose={onClose} />,
    );
    return { onClose, onPick };
  };

  it("takes the keyboard as soon as it opens", () => {
    show();

    expect((document.activeElement as HTMLElement).textContent).toContain("Nuevo documento");
  });

  it("leaves out what does not apply here", () => {
    show();

    expect(screen.queryByRole("menuitem", { name: "Nueva carpeta" })).toBeNull();
    expect(screen.getAllByRole("menuitem")).toHaveLength(3);
  });

  it("walks its options with the arrows and wraps around", async () => {
    show();

    await userEvent.keyboard("{ArrowDown}");
    expect((document.activeElement as HTMLElement).textContent).toContain("Renombrar");

    await userEvent.keyboard("{ArrowUp}{ArrowUp}");
    expect((document.activeElement as HTMLElement).textContent).toContain("Borrar");
  });

  it("closes before doing the thing, so nothing is left hanging", async () => {
    const order: string[] = [];
    const onClose = vi.fn(() => order.push("closed"));
    const onPick = vi.fn(() => order.push("picked"));
    show(onClose, onPick);

    await userEvent.click(screen.getByRole("menuitem", { name: "Renombrar" }));

    expect(order).toEqual(["closed", "picked"]);
  });

  it("hands the focus back to what opened it, unless the choice put it in a field of its own", async () => {
    const opener = document.createElement("button");
    const field = document.createElement("input");
    document.body.append(opener, field);
    for (const typed of [false, true]) {
      opener.focus();
      const { unmount } = render(<Shown typed={typed} field={field} />);
      await userEvent.click(screen.getByRole("menuitem", { name: "Renombrar" }));
      expect(document.activeElement, `typed ${typed}`).toBe(typed ? field : opener);
      unmount();
    }

    // Closed by a press somewhere else that took the focus, the menu leaves it there.
    opener.focus();
    const { unmount } = render(<Shown typed={false} field={field} />);
    await act(async () => {
      await new Promise((done) => setTimeout(done, 0));
    });
    field.focus();
    await act(async () => {
      field.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(field);
    unmount();
    opener.remove();
    field.remove();
  });

  it("gives up on escape and on a click outside", async () => {
    const { onClose } = show();

    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();

    await userEvent.click(document.body);
    expect(onClose.mock.calls.length).toBeGreaterThan(1);
  });

  it("goes into a nested menu and comes back out", async () => {
    const land = vi.fn();
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        label="Opciones"
        onClose={vi.fn()}
        choices={[
          {
            key: "move",
            label: "Mover a…",
            into: {
              label: "Mover a",
              choices: [
                { key: "a", label: "trabajo", onPick: () => land("trabajo") },
                { key: "b", label: "personal", onPick: () => land("personal") },
              ],
            },
          },
        ]}
      />,
    );

    await userEvent.click(screen.getByRole("menuitem", { name: /Mover a…/ }));

    expect(screen.getByRole("menu", { name: "Mover a" })).toBeTruthy();
    expect(screen.getByRole("menuitem", { name: "trabajo" })).toBeTruthy();

    await userEvent.keyboard("{Escape}");

    expect(screen.getByRole("menu", { name: "Opciones" })).toBeTruthy();
  });

  it("lands on the first destination, not on the way back", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        label="Opciones"
        onClose={vi.fn()}
        choices={[
          {
            key: "move",
            label: "Mover a…",
            into: {
              label: "Mover a",
              choices: [
                { key: "a", label: "trabajo", onPick: vi.fn() },
                { key: "b", label: "personal", onPick: vi.fn() },
              ],
            },
          },
        ]}
      />,
    );

    await userEvent.click(screen.getByRole("menuitem", { name: /Mover a…/ }));

    expect((document.activeElement as HTMLElement).textContent).toContain("trabajo");
  });

  it("picks a destination from the nested menu", async () => {
    const land = vi.fn();
    const onClose = vi.fn();
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        label="Opciones"
        onClose={onClose}
        choices={[
          {
            key: "move",
            label: "Mover a…",
            into: {
              label: "Mover a",
              choices: [{ key: "a", label: "trabajo", onPick: () => land("trabajo") }],
            },
          },
        ]}
      />,
    );

    await userEvent.click(screen.getByRole("menuitem", { name: /Mover a…/ }));
    await userEvent.click(screen.getByRole("menuitem", { name: "trabajo" }));

    expect(land).toHaveBeenCalledWith("trabajo");
    expect(onClose).toHaveBeenCalled();
  });

  it("names itself for whoever cannot see where it came from", () => {
    show();

    expect(screen.getByRole("menu", { name: "Opciones" })).toBeTruthy();
  });
});

describe("a menu of blocks", () => {
  it("shows each block's icon and its mark, and how the keys walk it", () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={[{ key: "h1", label: "Título", glyph: "heading1", mark: "#", onPick: vi.fn() }]}
        label="Bloques"
        find="Busca un bloque"
        foot="↑ ↓ para moverte"
        onClose={vi.fn()}
      />,
    );
    const item = screen.getByRole("menuitem");
    expect(item.querySelector("svg.glyph")).not.toBeNull();
    expect(item.textContent).toContain("#");
    expect(screen.getByText("↑ ↓ para moverte")).toBeTruthy();
  });

  const sought = () => {
    const onClose = vi.fn();
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={[{ key: "h1", label: "Título", onPick: vi.fn() }]}
        label="Bloques"
        find="Busca un bloque"
        onClose={onClose}
      />,
    );
    return { onClose, field: screen.getByRole("searchbox") };
  };

  it("hands back what was typed in its search when put away, so those words are not lost", async () => {
    const { onClose, field } = sought();
    await userEvent.type(field, "2 = 5");
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenLastCalledWith("2 = 5");
  });

  it("closes on a space right after the slash, a sentence going on", async () => {
    const { onClose, field } = sought();
    await userEvent.type(field, " ");
    expect(onClose).toHaveBeenCalledWith(" ");
  });

  it("opens the only list found with Enter, and its first choice is then a key away", async () => {
    const step = vi.fn();
    render(
      <Menu
        at={{ x: 10, y: 10 }}
        choices={[
          { key: "h1", label: "Título", onPick: vi.fn() },
          {
            key: "step",
            label: "Un paso",
            into: { label: "Un paso", choices: [{ key: "1", label: "1. Pan", onPick: step }] },
          },
        ]}
        label="Bloques"
        find="Busca un bloque"
        onClose={vi.fn()}
      />,
    );
    await userEvent.type(screen.getByRole("searchbox"), "paso{Enter}");
    await userEvent.click(screen.getByRole("menuitem", { name: "1. Pan" }));
    expect(step).toHaveBeenCalled();
  });
});

const nested = (land = vi.fn()): Choice[] => [
  { key: "a", label: "Alfa", onPick: vi.fn() },
  {
    key: "move",
    label: "Mover a",
    into: {
      label: "Destinos",
      choices: [
        { key: "w", label: "trabajo", onPick: () => land("trabajo") },
        { key: "p", label: "personal", onPick: () => land("personal") },
      ],
    },
  },
  { key: "b", label: "Beta", onPick: vi.fn() },
];

const active = () => document.activeElement as HTMLElement;

describe("walking a menu without search by keyboard", () => {
  const show = (onClose = vi.fn()) => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={onClose} />);
    return onClose;
  };

  it("wraps the arrows round from the last row to the first", async () => {
    show();

    await userEvent.keyboard("{ArrowUp}");
    expect(active().textContent).toContain("Beta");
    await userEvent.keyboard("{ArrowDown}");
    expect(active().textContent).toContain("Alfa");
  });

  it("jumps to the first and last row with Home and End", async () => {
    show();

    await userEvent.keyboard("{End}");
    expect(active().textContent).toContain("Beta");
    await userEvent.keyboard("{Home}");
    expect(active().textContent).toContain("Alfa");
  });

  it("jumps to the next row starting with the letter typed", async () => {
    show();

    await userEvent.keyboard("b");
    expect(active().textContent).toContain("Beta");
    await userEvent.keyboard("m");
    expect(active().textContent).toContain("Mover a");
  });

  it("roves the focus over the rows, one of them in the tab order", async () => {
    show();
    await userEvent.keyboard("{ArrowDown}");

    const items = screen.getAllByRole("menuitem");
    expect(items.map((one) => one.tabIndex)).toEqual([-1, 0, -1]);
    expect(active()).toBe(items[1]);
  });

  it("closes on Escape with nothing typed to hand back", async () => {
    const onClose = show();

    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledWith("");
  });
});

describe("walking a menu with search by keyboard", () => {
  const show = (onClose = vi.fn()) => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Bloques"
        find="Busca"
        onClose={onClose}
      />,
    );
    return onClose;
  };

  const marked = () =>
    screen
      .getAllByRole("menuitem")
      .filter((one) => one.hasAttribute("data-active"))
      .map((one) => one.textContent);

  it("keeps the focus in the search while the arrows move the mark", async () => {
    show();
    const field = screen.getByRole("searchbox");
    expect(active()).toBe(field);
    expect(marked()[0]).toContain("Alfa");

    await userEvent.keyboard("{ArrowDown}");
    expect(active()).toBe(field);
    expect(marked()[0]).toContain("Mover a");
    expect(field.getAttribute("aria-activedescendant")).toBe(screen.getAllByRole("menuitem")[1].id);

    await userEvent.keyboard("{ArrowUp}{ArrowUp}");
    expect(marked()[0]).toContain("Beta");
  });

  it("sends the letters to the words and not to a jump", async () => {
    show();

    await userEvent.keyboard("be");
    expect((screen.getByRole("searchbox") as HTMLInputElement).value).toBe("be");
    expect(screen.getAllByRole("menuitem")).toHaveLength(1);
    expect(active()).toBe(screen.getByRole("searchbox"));
  });

  it("marks the row Enter takes as what is typed narrows the list", async () => {
    show();

    await userEvent.type(screen.getByRole("searchbox"), "bet");

    expect(marked()).toEqual(["Beta"]);
  });

  it("hands back what was typed when Escape puts it away", async () => {
    const onClose = show();

    await userEvent.type(screen.getByRole("searchbox"), "ab");
    await userEvent.keyboard("{Escape}");

    expect(onClose).toHaveBeenLastCalledWith("ab");
  });

  it("picks the marked row with Enter", async () => {
    const onPick = vi.fn();
    const onClose = vi.fn();
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={[
          { key: "a", label: "Alfa", onPick: vi.fn() },
          { key: "b", label: "Beta", onPick },
        ]}
        label="Bloques"
        find="Busca"
        onClose={onClose}
      />,
    );

    await userEvent.keyboard("{ArrowDown}{Enter}");

    expect(onPick).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });
});

describe("rows held under a heading", () => {
  it("sit in a fieldset named by the heading, one for each run of the same group", () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        label="Bloques"
        onClose={vi.fn()}
        choices={[
          { key: "a", label: "Alfa", group: "Texto" },
          { key: "b", label: "Beta", group: "Texto" },
          { key: "c", label: "Gamma", group: "Listas" },
          { key: "d", label: "Delta" },
        ]}
      />,
    );

    const texto = screen.getByRole("group", { name: "Texto" });
    expect(texto.tagName).toBe("FIELDSET");
    expect(within(texto).getAllByRole("menuitem")).toHaveLength(2);
    expect(texto.textContent).toContain("Texto");
    const listas = screen.getByRole("group", { name: "Listas" });
    expect(within(listas).getAllByRole("menuitem")).toHaveLength(1);
    expect(screen.getAllByRole("group")).toHaveLength(2);
    expect(screen.getAllByRole("menuitem")).toHaveLength(4);
  });
});

describe("a list opened beside its row", () => {
  it("opens a second menu on click, focused on its first row", async () => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("menuitem", { name: /Mover a/ }));

    expect(screen.getAllByRole("menu")).toHaveLength(2);
    expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
    expect(active().textContent).toContain("trabajo");
    expect(screen.getByRole("menuitem", { name: /Mover a/ }).getAttribute("aria-expanded")).toBe(
      "true",
    );
  });

  it("opens with ArrowRight, and ArrowLeft closes it and gives the focus back to the row", async () => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);
    await userEvent.keyboard("{ArrowDown}{ArrowRight}");

    expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
    expect(active().textContent).toContain("trabajo");

    await userEvent.keyboard("{ArrowLeft}");

    expect(screen.queryByRole("menu", { name: "Destinos" })).toBeNull();
    expect(active().textContent).toContain("Mover a");
  });

  it("closes on Escape without closing the menu, and walks its own rows", async () => {
    const onClose = vi.fn();
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={onClose} />);
    await userEvent.keyboard("{ArrowDown}{ArrowRight}{ArrowDown}");
    expect(active().textContent).toContain("personal");

    await userEvent.keyboard("{Escape}");

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.queryByRole("menu", { name: "Destinos" })).toBeNull();
    expect(screen.getByRole("menu", { name: "Opciones" })).toBeTruthy();
  });

  it("gives the focus back to the search field of a search menu", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Bloques"
        find="Busca"
        onClose={vi.fn()}
      />,
    );
    await userEvent.keyboard("{ArrowDown}{ArrowRight}");
    expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
    expect(active().textContent).toContain("trabajo");

    await userEvent.keyboard("{ArrowLeft}");

    expect(active()).toBe(screen.getByRole("searchbox"));
    expect(screen.queryByRole("menu", { name: "Destinos" })).toBeNull();
  });

  it("opens with Enter on the marked row of a search menu", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Bloques"
        find="Busca"
        onClose={vi.fn()}
      />,
    );
    await userEvent.type(screen.getByRole("searchbox"), "mover{Enter}");

    expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
  });

  it("opens after a moment of hovering its row", async () => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);

    await userEvent.hover(screen.getByRole("menuitem", { name: /Mover a/ }));

    expect(await screen.findByRole("menu", { name: "Destinos" })).toBeTruthy();
  });

  it("moves the focus into a list opened by hovering when ArrowRight is pressed", () => {
    vi.useFakeTimers();
    try {
      render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);
      fireEvent.mouseMove(screen.getByRole("menuitem", { name: /Mover a/ }));
      act(() => {
        vi.advanceTimersByTime(200);
      });
      expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
      expect(active().textContent).toContain("Mover a");

      fireEvent.keyDown(active(), { key: "ArrowRight" });

      expect(active().textContent).toContain("trabajo");
    } finally {
      vi.useRealTimers();
    }
  });

  it("picks in it, calling the choice and putting the whole menu away", async () => {
    const land = vi.fn();
    const onClose = vi.fn();
    render(
      <Menu at={{ x: 20, y: 30 }} choices={nested(land)} label="Opciones" onClose={onClose} />,
    );
    await userEvent.keyboard("{ArrowDown}{ArrowRight}{ArrowDown}{Enter}");

    expect(land).toHaveBeenCalledWith("personal");
    expect(onClose).toHaveBeenCalled();
  });

  it("is found by the labels of its children when searching", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Bloques"
        find="Busca"
        onClose={vi.fn()}
      />,
    );

    await userEvent.type(screen.getByRole("searchbox"), "trabajo");

    expect(screen.getAllByRole("menuitem")).toHaveLength(1);
    expect(screen.getByRole("menuitem").textContent).toContain("Mover a");
  });
});

describe("shortcut tiles, and the message when nothing matches", () => {
  const show = (onClose = vi.fn(), onPick = vi.fn()) => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={[{ key: "a", label: "Alfa", onPick: vi.fn() }]}
        row={[
          { key: "t1", label: "Titulo", icon: "T", onPick },
          { key: "t2", label: "Lista", icon: "L", onPick: vi.fn() },
        ]}
        none={(sought) => `Nada se llama ${sought}`}
        label="Bloques"
        find="Busca"
        onClose={onClose}
      />,
    );
    return { onClose, onPick };
  };

  it("are plain buttons over the list while nothing is sought", () => {
    show();

    expect(screen.getByRole("button", { name: "Titulo" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Lista" })).toBeTruthy();
    expect(screen.getAllByRole("menuitem")).toHaveLength(1);
  });

  it("step aside while something is sought", async () => {
    show();

    await userEvent.type(screen.getByRole("searchbox"), "alf");

    expect(screen.queryByRole("button", { name: "Titulo" })).toBeNull();
  });

  it("close the menu and pick when pressed", async () => {
    const order: string[] = [];
    const { onClose, onPick } = show(
      vi.fn(() => order.push("closed")),
      vi.fn(() => order.push("picked")),
    );

    await userEvent.click(screen.getByRole("button", { name: "Titulo" }));

    expect(onClose).toHaveBeenCalled();
    expect(onPick).toHaveBeenCalled();
    expect(order).toEqual(["closed", "picked"]);
  });

  it("says so in a status when nothing matches, naming what was sought", async () => {
    show();
    expect(screen.queryByRole("status")).toBeNull();

    await userEvent.type(screen.getByRole("searchbox"), "zzz");

    expect(screen.getByRole("status").textContent).toBe("Nada se llama zzz");
    expect(screen.queryAllByRole("menuitem")).toHaveLength(0);
  });
});

describe("a menu laid once", () => {
  const WIDE = 210;
  const ROW = 30;

  beforeEach(() => {
    Object.defineProperty(window, "innerHeight", { configurable: true, value: 800 });
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 1200 });
    Object.defineProperty(HTMLElement.prototype, "scrollHeight", {
      configurable: true,
      get() {
        const own = this as HTMLElement;
        return own.getAttribute("role") === "menu"
          ? own.querySelectorAll("[role=menuitem]").length * ROW + 40
          : 0;
      },
    });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
      () => ({ top: 0, bottom: 0, left: 0, right: WIDE, width: WIDE, height: 0 }) as DOMRect,
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
    Reflect.deleteProperty(HTMLElement.prototype, "scrollHeight");
  });

  const many = Array.from(
    { length: 12 },
    (_, at): Choice => ({
      key: `k${at}`,
      label: `${at % 2 ? "Beta" : "Alfa"} ${at}`,
      onPick: vi.fn(),
    }),
  );

  it("does not move when the filter shortens it, a menu above its line stays touching it", async () => {
    render(
      <Menu
        at={{ x: 100, y: 780, top: 760 }}
        choices={many}
        label="Bloques"
        find="Busca"
        onClose={vi.fn()}
      />,
    );
    const menu = screen.getByRole("menu");
    const before = { left: menu.style.left, bottom: menu.style.bottom, top: menu.style.top };
    expect(before.bottom).toBe(`${800 - 756}px`);

    await userEvent.type(screen.getByRole("searchbox"), "beta");

    expect(screen.getAllByRole("menuitem")).toHaveLength(6);
    const after = { left: menu.style.left, bottom: menu.style.bottom, top: menu.style.top };
    expect(after).toEqual(before);
  });

  it("is laid again when the window is resized", () => {
    render(<Menu at={{ x: 1100, y: 100 }} choices={many} label="Bloques" onClose={vi.fn()} />);
    const menu = screen.getByRole("menu");
    expect(menu.style.left).toBe(`${1200 - WIDE - 6}px`);

    Object.defineProperty(window, "innerWidth", { configurable: true, value: 900 });
    act(() => {
      window.dispatchEvent(new Event("resize"));
    });

    expect(menu.style.left).toBe(`${900 - WIDE - 6}px`);
  });

  it("is capped by `tallest`, so a long list scrolls", () => {
    render(
      <Menu
        at={{ x: 100, y: 100 }}
        choices={many}
        label="Bloques"
        tallest={200}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByRole("menu").style.maxHeight).toBe("200px");
  });

  it("grows to its content when `tallest` is not reached", () => {
    render(
      <Menu
        at={{ x: 100, y: 100 }}
        choices={many}
        label="Bloques"
        tallest={2000}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByRole("menu").style.maxHeight).toBe(`${12 * ROW + 40}px`);
  });

  it("takes the width it is given", () => {
    render(
      <Menu at={{ x: 100, y: 100 }} choices={many} label="Bloques" wide={320} onClose={vi.fn()} />,
    );

    expect(screen.getByRole("menu").style.width).toBe("320px");
  });
});

describe("the width a menu is laid by", () => {
  const offsetWidth = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetWidth");
  afterEach(() => {
    vi.restoreAllMocks();
    if (offsetWidth) Object.defineProperty(HTMLElement.prototype, "offsetWidth", offsetWidth);
  });

  it("is its layout width when that is larger than the transformed one", () => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 900 });
    Object.defineProperty(window, "innerHeight", { configurable: true, value: 800 });
    Object.defineProperty(HTMLElement.prototype, "offsetWidth", {
      configurable: true,
      get() {
        return (this as HTMLElement).getAttribute("role") === "menu" ? 320 : 0;
      },
    });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
      () => ({ top: 0, bottom: 0, left: 0, right: 313.6, width: 313.6, height: 0 }) as DOMRect,
    );

    render(<Menu at={{ x: 700, y: 100 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);

    const menu = screen.getByRole("menu");
    expect(Number.parseFloat(menu.style.left) + 320).toBe(894);
  });

  it("starts a flyout unseen but not hidden, so it can still take the focus", async () => {
    const seen: string[] = [];
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);
    const row = screen.getByRole("menuitem", { name: /Mover a/ });
    vi.spyOn(row, "getBoundingClientRect").mockImplementation(() => {
      const flyout = document.querySelector<HTMLElement>('[role=menu][aria-label="Destinos"]');
      seen.push(`${flyout?.style.opacity}|${flyout?.style.visibility}`);
      return { top: 0, bottom: 0, left: 0, right: 0, width: 0, height: 0 } as DOMRect;
    });

    await userEvent.click(row);

    expect(seen[0]).toBe("0|");
    expect(screen.getByRole("menu", { name: "Destinos" }).style.opacity).toBe("");
  });
});

const withAlerts = (land = vi.fn()): Choice[] => [
  { key: "a", label: "Alfa", onPick: vi.fn() },
  {
    key: "alerts",
    label: "Avisos",
    into: {
      label: "Tipos",
      choices: [
        { key: "note", label: "Nota", onPick: () => land("Nota") },
        { key: "warn", label: "Advertencia", onPick: () => land("Advertencia") },
      ],
    },
  },
];

const searched = (land = vi.fn()) =>
  render(
    <Menu
      at={{ x: 20, y: 30 }}
      choices={withAlerts(land)}
      label="Bloques"
      find="Busca"
      onClose={vi.fn()}
    />,
  );

describe("a parent row found by the words of one of its children", () => {
  it("opens on that child with Enter, and Enter again picks it, not the first one", async () => {
    const land = vi.fn();
    searched(land);

    await userEvent.type(screen.getByRole("searchbox"), "advertencia{Enter}");

    expect(screen.getByRole("menu", { name: "Tipos" })).toBeTruthy();
    expect(active().textContent).toContain("Advertencia");

    await userEvent.keyboard("{Enter}");

    expect(land).toHaveBeenCalledTimes(1);
    expect(land).toHaveBeenCalledWith("Advertencia");
  });

  it("opens on that child with ArrowRight from the search", async () => {
    searched();

    await userEvent.type(screen.getByRole("searchbox"), "advertencia{ArrowRight}");

    expect(screen.getByRole("menu", { name: "Tipos" })).toBeTruthy();
    expect(active().textContent).toContain("Advertencia");
  });

  it("opens on the first child when the words match the parent's own label only", async () => {
    searched();

    await userEvent.type(screen.getByRole("searchbox"), "avisos{Enter}");

    expect(active().textContent).toContain("Nota");
  });

  it("opens on the first child when nothing is sought", async () => {
    searched();

    await userEvent.keyboard("{ArrowDown}{ArrowRight}");

    expect(screen.getByRole("menu", { name: "Tipos" })).toBeTruthy();
    expect(active().textContent).toContain("Nota");
  });
});

describe("the pointer crossing from a parent row to its list", () => {
  const crossing = () => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);
    fireEvent.mouseMove(screen.getByRole("menuitem", { name: /Mover a/ }));
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(screen.getByRole("menu", { name: "Destinos" })).toBeTruthy();
    fireEvent.mouseMove(screen.getByRole("menuitem", { name: "Beta" }));
  };

  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("keeps the list open once the pointer enters it, though a sibling row asked to close it", () => {
    crossing();

    fireEvent.pointerEnter(screen.getByRole("menu", { name: "Destinos" }));
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(screen.queryByRole("menu", { name: "Destinos" })).toBeTruthy();
  });

  it("closes the list when the pointer never reaches it", () => {
    crossing();

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(screen.queryByRole("menu", { name: "Destinos" })).toBeNull();
  });
});

describe("the active row clear of the sticky heading and foot", () => {
  const own = (name: string) => Object.getOwnPropertyDescriptor(HTMLElement.prototype, name);
  const originals = { offsetHeight: own("offsetHeight"), scrollIntoView: own("scrollIntoView") };
  const scrolled = vi.fn();

  const restore = (name: "offsetHeight" | "scrollIntoView") => {
    const was = originals[name];
    if (was) Object.defineProperty(HTMLElement.prototype, name, was);
    else Reflect.deleteProperty(HTMLElement.prototype, name);
  };

  beforeEach(() => {
    scrolled.mockClear();
    Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
      configurable: true,
      get() {
        const named = (this as HTMLElement).className;
        if (named.includes("sticky -top-1")) return 44;
        if (named.includes("sticky -bottom-1")) return 28;
        return 0;
      },
    });
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrolled,
    });
  });

  afterEach(() => {
    restore("offsetHeight");
    restore("scrollIntoView");
  });

  const lastCallOn = (row: HTMLElement) => {
    const at = scrolled.mock.contexts.lastIndexOf(row);
    return at < 0 ? undefined : scrolled.mock.calls[at][0];
  };

  it("pads the card by both heights and brings the newly active row into view", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Bloques"
        find="Busca"
        foot="Flechas"
        onClose={vi.fn()}
      />,
    );
    const card = screen.getByRole("menu", { name: "Bloques" });
    scrolled.mockClear();

    await userEvent.keyboard("{ArrowDown}");

    expect(card.style.scrollPaddingTop).toBe("44px");
    expect(card.style.scrollPaddingBottom).toBe("28px");
    expect(lastCallOn(screen.getByRole("menuitem", { name: /Mover a/ }))).toEqual({
      block: "nearest",
    });
  });

  it("leaves a row marked by the pointer alone, and scrolls to one marked by a key", async () => {
    render(<Menu at={{ x: 20, y: 30 }} choices={nested()} label="Opciones" onClose={vi.fn()} />);
    const row = screen.getByRole("menuitem", { name: "Beta" });
    scrolled.mockClear();

    fireEvent.mouseMove(row);

    expect(row.hasAttribute("data-active")).toBe(true);
    expect(scrolled.mock.contexts).not.toContain(row);

    await userEvent.keyboard("{ArrowUp}{ArrowDown}");

    expect(lastCallOn(row)).toEqual({ block: "nearest" });
  });

  it("does the same in a menu without search, where the row is focused without scrolling", async () => {
    render(
      <Menu
        at={{ x: 20, y: 30 }}
        choices={nested()}
        label="Opciones"
        foot="Flechas"
        onClose={vi.fn()}
      />,
    );
    const card = screen.getByRole("menu", { name: "Opciones" });
    scrolled.mockClear();

    await userEvent.keyboard("{ArrowDown}");

    expect(card.style.scrollPaddingTop).toBe("0px");
    expect(card.style.scrollPaddingBottom).toBe("28px");
    expect(lastCallOn(screen.getByRole("menuitem", { name: /Mover a/ }))).toEqual({
      block: "nearest",
    });
  });
});

describe("switching from one list to another", () => {
  const twins = (): Choice[] => [
    {
      key: "a",
      label: "Primera",
      into: {
        label: "Lista A",
        choices: [
          { key: "a1", label: "A uno", onPick: vi.fn() },
          { key: "a2", label: "A dos", onPick: vi.fn() },
        ],
      },
    },
    {
      key: "b",
      label: "Segunda",
      into: {
        label: "Lista B",
        choices: [
          { key: "b1", label: "B uno", onPick: vi.fn() },
          { key: "b2", label: "B dos", onPick: vi.fn() },
        ],
      },
    },
  ];
  const show = () =>
    render(<Menu at={{ x: 20, y: 30 }} choices={twins()} label="Opciones" onClose={vi.fn()} />);

  it("starts the second on its first row, not where the first was left", async () => {
    show();

    await userEvent.keyboard("{ArrowRight}{ArrowDown}");
    expect(active().textContent).toContain("A dos");
    await userEvent.keyboard("{ArrowLeft}{ArrowDown}{ArrowRight}");

    expect(screen.getByRole("menu", { name: "Lista B" })).toBeTruthy();
    expect(screen.queryByRole("menu", { name: "Lista A" })).toBeNull();
    expect(active().textContent).toContain("B uno");
  });

  it("shows the hovered one with no mark left over from the other", () => {
    vi.useFakeTimers();
    try {
      show();
      fireEvent.mouseMove(screen.getByRole("menuitem", { name: /Primera/ }));
      act(() => {
        vi.advanceTimersByTime(200);
      });
      expect(screen.getByRole("menu", { name: "Lista A" })).toBeTruthy();

      fireEvent.mouseMove(screen.getByRole("menuitem", { name: /Segunda/ }));
      act(() => {
        vi.advanceTimersByTime(200);
      });

      expect(screen.queryByRole("menu", { name: "Lista A" })).toBeNull();
      const second = screen.getByRole("menu", { name: "Lista B" });
      expect(second.querySelector("[data-active]")).toBeNull();
      expect(within(second).getAllByRole("menuitem")).toHaveLength(2);
    } finally {
      vi.useRealTimers();
    }
  });
});
