import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { List, Task } from "../core";
import Matrix from "../ui/Matrix";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    Promise.resolve(
      cmd === "icons"
        ? [
            ["home", "🏠"],
            ["work", "💼"],
          ]
        : null,
    ),
}));

const lists: List[] = [
  { id: "L1", name: "Work", icon: "work", archived: false, order: "a0" },
  { id: "L2", name: "Home", icon: "home", archived: false, order: "a1" },
];

const task = (id: string, title: string, priority: Task["priority"], list?: string): Task =>
  ({ id, title, status: "open", priority, order: id, list }) as Task;

const tasks: Task[] = [
  task("1", "close the quarter", "do", "L1"),
  task("2", "plan the Rust course", "decide", "L1"),
  task("3", "answer the supplier survey", "delegate", "L1"),
  task("4", "compare the tariffs again", "wont", "L2"),
  task("5", "quote the move", "unset", "L2"),
  task("6", "renew the passport", "unset", "L1"),
];

const widen = (px: number) => {
  Object.defineProperty(window, "innerWidth", { value: px, configurable: true, writable: true });
};

const show = (extra: Partial<React.ComponentProps<typeof Matrix>> = {}) =>
  render(
    <Matrix
      tasks={tasks}
      lists={lists}
      onPlace={vi.fn()}
      onOpen={vi.fn()}
      onDiscardAll={vi.fn()}
      {...extra}
    />,
  );

const quadrant = (name: string) => screen.getByRole("group", { name });

const openTray = async () =>
  userEvent.click(await screen.findByRole("button", { name: /What is left to classify/ }));

const carried = () => {
  Element.prototype.setPointerCapture = () => {};
  Element.prototype.releasePointerCapture = () => {};
};

const drag = async (title: string, onto: HTMLElement) => {
  carried();
  const card = screen.getByRole("button", { name: title });
  document.elementFromPoint = () => onto;

  await act(async () => {
    fireEvent.pointerDown(card, { button: 0, pointerId: 1, clientX: 10, clientY: 10 });
  });
  await act(async () => {
    fireEvent.pointerMove(card, { pointerId: 1, clientX: 400, clientY: 300 });
  });
  await act(async () => {
    fireEvent.pointerMove(card, { pointerId: 1, clientX: 402, clientY: 302 });
    fireEvent.pointerUp(card, { pointerId: 1, clientX: 402, clientY: 302 });
  });
};

const tap = async (title: string) => {
  carried();
  const card = screen.getByRole("button", { name: title });
  document.elementFromPoint = () => card;

  await act(async () => {
    fireEvent.pointerDown(card, { button: 0, pointerId: 1, clientX: 10, clientY: 10 });
    fireEvent.pointerUp(card, { pointerId: 1, clientX: 11, clientY: 11 });
  });
};

describe("the matrix", () => {
  it("puts every task in the quadrant it was placed in", () => {
    widen(1500);
    show();

    expect(within(quadrant("Do")).getByText("close the quarter")).toBeTruthy();
    expect(within(quadrant("Decide")).getByText("plan the Rust course")).toBeTruthy();
    expect(within(quadrant("Delegate")).getByText("answer the supplier survey")).toBeTruthy();
    expect(within(quadrant("Drop")).getByText("compare the tariffs again")).toBeTruthy();
  });

  it("keeps what nobody placed out of the four, in the tray", async () => {
    widen(1500);
    show();
    await openTray();

    const tray = screen.getByRole("complementary", { name: "Unclassified" });
    expect(within(tray).getByText("quote the move")).toBeTruthy();
    expect(within(tray).getByText("renew the passport")).toBeTruthy();
  });

  it("names both axes so the quadrants explain themselves", () => {
    widen(1500);
    show();

    for (const axis of ["Urgent", "Not urgent", "Important", "Not important"]) {
      expect(screen.getByText(axis)).toBeTruthy();
    }
  });

  it("places a task where it was dropped", async () => {
    widen(1500);
    const onPlace = vi.fn();
    show({ onPlace });
    await openTray();

    await drag("renew the passport", quadrant("Do"));

    expect(onPlace).toHaveBeenCalledWith("6", "do");
  });

  it("takes a task back to the tray when it is dropped there", async () => {
    widen(1500);
    const onPlace = vi.fn();
    show({ onPlace });

    await openTray();
    await drag("close the quarter", screen.getByRole("complementary", { name: "Unclassified" }));

    expect(onPlace).toHaveBeenCalledWith("1", "unset");
  });

  it("only offers to empty the quadrant that means it", async () => {
    widen(1500);
    const onDiscardAll = vi.fn();
    show({ onDiscardAll });

    expect(
      within(quadrant("Do")).queryByRole("button", { name: "I won't do any of them" }),
    ).toBeNull();
    await userEvent.click(
      within(quadrant("Drop")).getByRole("button", { name: "I won't do any of them" }),
    );

    expect(onDiscardAll).toHaveBeenCalledWith(["4"]);
  });

  it("stays out of the way until it is asked for", async () => {
    widen(1500);
    show();
    expect(screen.queryByRole("complementary", { name: "Unclassified" })).toBeNull();

    await openTray();

    expect(screen.getByRole("complementary", { name: "Unclassified" })).toBeTruthy();
  });

  it("comes when it is asked for on a narrow window too", async () => {
    widen(1100);
    show();

    await openTray();

    expect(screen.getByRole("complementary", { name: "Unclassified" })).toBeTruthy();
  });

  it("narrows the tray to the lists asked for, and leaves the quadrants whole", async () => {
    widen(1500);
    show();
    await openTray();
    await userEvent.click(screen.getByRole("button", { name: /Only in/ }));
    await userEvent.click(await screen.findByRole("checkbox", { name: /Home/ }));

    const tray = screen.getByRole("complementary", { name: "Unclassified" });
    expect(within(tray).getByText("quote the move")).toBeTruthy();
    expect(within(tray).queryByText("renew the passport")).toBeNull();
    expect(within(quadrant("Do")).getByText("close the quarter")).toBeTruthy();
  });

  it("opens the task when it is clicked, without moving it", async () => {
    widen(1500);
    const onOpen = vi.fn();
    const onPlace = vi.fn();
    show({ onOpen, onPlace });

    await tap("close the quarter");

    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "1" }));
    expect(onPlace).not.toHaveBeenCalled();
  });

  it("leaves a task where it was when it is dropped outside every zone", async () => {
    widen(1500);
    const onPlace = vi.fn();
    show({ onPlace });
    await openTray();
    Element.prototype.setPointerCapture = () => {};
    const card = screen.getByRole("button", { name: "renew the passport" });
    document.elementFromPoint = () => document.body;

    await act(async () => {
      fireEvent.pointerDown(card, { button: 0, pointerId: 1, clientX: 10, clientY: 10 });
    });
    await act(async () => {
      fireEvent.pointerMove(card, { pointerId: 1, clientX: 500, clientY: 500 });
      fireEvent.pointerUp(card, { pointerId: 1, clientX: 500, clientY: 500 });
    });

    expect(onPlace).not.toHaveBeenCalled();
  });

  it("does not place a task that never left its quadrant", async () => {
    widen(1500);
    const onPlace = vi.fn();
    show({ onPlace });

    await drag("close the quarter", quadrant("Do"));

    expect(onPlace).not.toHaveBeenCalled();
  });

  it("retires on its own when the window gets too narrow", async () => {
    widen(1500);
    show();
    await openTray();
    expect(screen.getByRole("complementary", { name: "Unclassified" })).toBeTruthy();

    await act(async () => {
      widen(1100);
      window.dispatchEvent(new Event("resize"));
    });

    expect(screen.queryByRole("complementary", { name: "Unclassified" })).toBeNull();
  });
});
