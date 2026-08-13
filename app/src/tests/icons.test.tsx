import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Lists from "../ui/Lists";
import type { List } from "../core";

const store = vi.hoisted(() => ({
  looks: [] as { id: string; icon?: string }[],
  made: [] as { name: string; icon?: string }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "icons":
        return Promise.resolve([
          ["home", "🏠"],
          ["work", "💼"],
          ["health", "🩺"],
        ]);
      case "list_look":
        store.looks.push({ id: String(args?.id), icon: args?.icon as string | undefined });
        return Promise.resolve({ id: args?.id, name: "Casa", order: "a0", icon: args?.icon });
      case "list_add":
        store.made.push({ name: String(args?.name), icon: args?.icon as string | undefined });
        return Promise.resolve({ id: "01L", name: args?.name, order: "a1", icon: args?.icon });
      default:
        return Promise.resolve(null);
    }
  },
}));

const lists: List[] = [
  { id: "01A", name: "Casa", order: "a0", icon: "home" },
  { id: "01B", name: "Trabajo", order: "a1" },
];

describe("icons on a list", () => {
  beforeEach(() => {
    store.looks = [];
    store.made = [];
  });

  const show = () =>
    render(
      <Lists
        lists={lists}
        counts={{ "01A": 2 }}
        onOpen={vi.fn()}
        onChanged={vi.fn()}
        onError={vi.fn()}
      />,
    );

  it("draws the one a list already carries", async () => {
    show();

    expect(await screen.findByText("🏠")).toBeTruthy();
  });

  it("shows a plain mark where there is none, rather than nothing to press", async () => {
    show();

    expect(await screen.findByLabelText("Icon of Trabajo")).toBeTruthy();
  });

  it("keeps the key, never the drawing", async () => {
    show();
    await userEvent.click(await screen.findByLabelText("Icon of Trabajo"));
    await userEvent.click(await screen.findByRole("button", { name: "work" }));

    await waitFor(() => expect(store.looks.length).toBe(1));
    expect(store.looks[0]).toEqual({ id: "01B", icon: "work" });
  });

  it("can take an icon back off", async () => {
    show();
    await userEvent.click(await screen.findByLabelText("Icon of Casa"));
    await userEvent.click(await screen.findByRole("button", { name: "No icon" }));

    await waitFor(() => expect(store.looks.length).toBe(1));
    expect(store.looks[0].icon).toBeUndefined();
  });

  it("carries the icon into a list being made", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: "New list" }));
    await userEvent.type(screen.getByLabelText("Name of the list"), "Salud");
    await userEvent.click(await screen.findByRole("button", { name: "health" }));
    await userEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(store.made.length).toBe(1));
    expect(store.made[0]).toEqual({ name: "Salud", icon: "health" });
  });

  it("refuses to make one with no name", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: "New list" }));
    await userEvent.click(screen.getByRole("button", { name: "Create" }));

    expect(store.made.length).toBe(0);
  });
});
