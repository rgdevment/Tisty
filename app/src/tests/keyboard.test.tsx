import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import TaskList from "../ui/TaskList";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (id: string, title: string): Task =>
  ({ id, title, status: "open", priority: 4, order: "a0", tags: [], reminders: [] }) as unknown as Task;

const three = [task("1", "pagar la luz"), task("2", "llamar al dentista"), task("3", "leer")];

const show = (onSelect = () => {}, onComplete?: (id: string) => void) =>
  render(
    <TaskList
      tasks={three}
      lists={[]}
      title="Open"
      centred
      onSelect={onSelect}
      onComplete={onComplete}
    />,
  );

const rows = () => screen.getAllByRole("listitem");

describe("moving through the list without a mouse", () => {
  /// The rows were plain divs with onClick: nothing in them took focus at all.
  it("lets a row take focus", async () => {
    show();

    await userEvent.tab();

    expect(document.activeElement).toBe(rows()[0]);
  });

  /// Ninety rows between the list and the sidebar is not navigation.
  it("offers one tab stop for the whole list", () => {
    show();

    expect(rows().filter((row) => row.getAttribute("tabindex") === "0").length).toBe(1);
  });

  it("walks down and up with the arrows", async () => {
    show();

    await userEvent.tab();
    await userEvent.keyboard("{ArrowDown}");
    expect(document.activeElement).toBe(rows()[1]);

    await userEvent.keyboard("{ArrowDown}");
    expect(document.activeElement).toBe(rows()[2]);

    await userEvent.keyboard("{ArrowUp}");
    expect(document.activeElement).toBe(rows()[1]);
  });

  it("stays put at either end instead of wrapping around", async () => {
    show();

    await userEvent.tab();
    await userEvent.keyboard("{ArrowUp}");

    expect(document.activeElement).toBe(rows()[0]);
  });

  it("opens the task on Enter", async () => {
    const opened = vi.fn();
    show(opened);

    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(opened).toHaveBeenCalledWith("1");
  });

  it("opens it on Space too", async () => {
    const opened = vi.fn();
    show(opened);

    await userEvent.tab();
    await userEvent.keyboard(" ");

    expect(opened).toHaveBeenCalledWith("1");
  });

  /// THE DANGEROUS ONE: the only focusable thing per row used to be the
  /// complete button, labelled with the task title. Whoever tabbed to what read
  /// like «the task» and pressed Enter CLOSED it — and the window has no undo.
  it("never lets Enter on a row complete the task", async () => {
    const opened = vi.fn();
    const completed = vi.fn();
    show(opened, completed);

    await userEvent.tab();
    expect((document.activeElement as HTMLElement).tagName).not.toBe("BUTTON");

    await userEvent.keyboard("{Enter}");

    expect(completed).not.toHaveBeenCalled();
    expect(opened).toHaveBeenCalled();
  });

  it("says what the circle does instead of naming the task", () => {
    show(() => {}, vi.fn());

    expect(screen.getByRole("button", { name: "Complete pagar la luz" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "pagar la luz" })).toBeNull();
  });

  it("keeps the circle clickable for whoever uses a mouse", async () => {
    const completed = vi.fn();
    show(() => {}, completed);

    await userEvent.click(screen.getByRole("button", { name: "Complete pagar la luz" }));

    expect(completed).toHaveBeenCalledWith("1");
  });

  /// Completing is the verb this product exists for, and until now it was
  /// mouse-only: the circle is out of the tab order on purpose, and the detail
  /// only offers «Not doing it», which discards.
  it("completes the focused task with Ctrl+Enter", async () => {
    const completed = vi.fn();
    show(() => {}, completed);

    await userEvent.tab();
    await userEvent.keyboard("{Control>}{Enter}{/Control}");

    expect(completed).toHaveBeenCalledWith("1");
  });

  /// Plain Enter still opens: the destructive-by-accident shape is what the
  /// whole keyboard work was about.
  it("does not complete on plain Enter", async () => {
    const opened = vi.fn();
    const completed = vi.fn();
    show(opened, completed);

    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(completed).not.toHaveBeenCalled();
    expect(opened).toHaveBeenCalled();
  });

  /// So a run of them can be ticked off without reaching for the mouse.
  it("moves to the next one after completing", async () => {
    show(() => {}, vi.fn());

    await userEvent.tab();
    await userEvent.keyboard("{Control>}{Enter}{/Control}");

    expect(document.activeElement).toBe(rows()[1]);
  });
});
