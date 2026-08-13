import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Menu, { type Choice } from "../ui/Menu";

const choices = (onPick = vi.fn()): Choice[] => [
  { key: "new", label: "Nuevo documento", icon: "+", onPick },
  { key: "folder", label: "Nueva carpeta", icon: "+", off: true, onPick },
  { key: "rename", label: "Renombrar", apart: true, onPick },
  { key: "drop", label: "Borrar", danger: true, apart: true, onPick },
];

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
