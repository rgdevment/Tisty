import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { List } from "../core";
import Only, { said } from "../ui/Only";
import { asView } from "../views";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    Promise.resolve(
      cmd === "icons"
        ? [
            ["work", "💼"],
            ["family", "👪"],
          ]
        : null,
    ),
}));

const lists: List[] = [
  { id: "01A", name: "Trabajo", order: "a0", icon: "work" },
  { id: "01B", name: "Personal", order: "a1" },
  { id: "01C", name: "Familia", order: "a2", icon: "family" },
];

describe("picking which lists to see", () => {
  const changed = vi.fn();

  beforeEach(() => changed.mockClear());

  const show = (chosen: string[] = []) =>
    render(<Only lists={lists} chosen={chosen} onChange={changed} />);

  const opened = async () => {
    await userEvent.click(screen.getByRole("button", { name: /Only in/ }));
  };

  it("offers every list as a box that can be ticked", async () => {
    show();
    await opened();

    expect(screen.getByRole("checkbox", { name: /Trabajo/ })).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: /Personal/ })).toBeTruthy();
    expect(screen.getByRole("checkbox", { name: /Familia/ })).toBeTruthy();
  });

  it("adds a list without dropping the ones already picked", async () => {
    show(["01A"]);
    await userEvent.click(screen.getByRole("button", { name: /Trabajo/ }));
    await userEvent.click(screen.getByRole("checkbox", { name: /Familia/ }));

    expect(changed).toHaveBeenCalledWith(["01A", "01C"]);
  });

  it("takes a list back off when its box is unticked", async () => {
    show(["01A", "01C"]);
    await userEvent.click(screen.getByRole("button", { name: /2 lists/ }));
    await userEvent.click(screen.getByRole("checkbox", { name: /Trabajo/ }));

    expect(changed).toHaveBeenCalledWith(["01C"]);
  });

  it("stays open while boxes are ticked, so several can be picked at once", async () => {
    show();
    await opened();
    await userEvent.click(screen.getByRole("checkbox", { name: /Trabajo/ }));

    expect(screen.getByRole("checkbox", { name: /Familia/ })).toBeTruthy();
  });

  it("clears the whole filter in one go", async () => {
    show(["01A", "01B"]);
    await userEvent.click(screen.getByRole("button", { name: /2 lists/ }));
    await userEvent.click(screen.getByRole("button", { name: "Every list" }));

    expect(changed).toHaveBeenCalledWith([]);
  });

  it("offers nothing to clear when nothing is filtered", async () => {
    show();
    await opened();

    expect(screen.queryByRole("button", { name: "Every list" })).toBeNull();
  });

  it("shuts on Escape", async () => {
    show();
    await opened();
    await userEvent.keyboard("{Escape}");

    expect(screen.queryByRole("checkbox", { name: /Trabajo/ })).toBeNull();
  });

  it("keeps out of the way when there are no lists at all", () => {
    render(<Only lists={[]} chosen={[]} onChange={changed} />);

    expect(screen.queryByRole("button")).toBeNull();
  });
});

describe("what the button says", () => {
  it("invites when no list is picked", () => {
    expect(said(lists, [])).toBe("Only in");
  });

  it("names the one list, which beats counting to one", () => {
    expect(said(lists, ["01B"])).toBe("Personal");
  });

  it("counts them once there are several", () => {
    expect(said(lists, ["01A", "01C"])).toBe("2 lists");
  });

  it("falls back to the invitation when the list is gone", () => {
    expect(said(lists, ["01Z"])).toBe("Only in");
  });
});

describe("the filter reaching the store", () => {
  it("narrows the slice instead of replacing it", () => {
    expect(asView({ named: "tasks", slice: "today", lists: ["01A"] })).toEqual({
      window: "today",
      lists: ["01A"],
    });
  });

  it("asks for nothing extra when no list is picked", () => {
    expect(asView({ named: "tasks", slice: "all", lists: [] })).toEqual({});
  });

  it("carries every picked list through", () => {
    expect(asView({ named: "tasks", slice: "repeating", lists: ["01A", "01B"] })).toEqual({
      repeating: true,
      lists: ["01A", "01B"],
    });
  });
});
