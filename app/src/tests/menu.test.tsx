import { act, render, screen } from "@testing-library/react";
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
    const Shown = ({ typed }: { typed: boolean }) => {
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
    for (const typed of [false, true]) {
      opener.focus();
      const { unmount } = render(<Shown typed={typed} />);
      await userEvent.click(screen.getByRole("menuitem", { name: "Renombrar" }));
      expect(document.activeElement, `typed ${typed}`).toBe(typed ? field : opener);
      unmount();
    }

    // Closed by a press somewhere else that took the focus, the menu leaves it there.
    opener.focus();
    const { unmount } = render(<Shown typed={false} />);
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
