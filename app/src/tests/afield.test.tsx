import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DateSpec, Task, Whole } from "../core";
import { whenLabel } from "../format";
import { fill } from "../locales";
import TaskList from "../ui/TaskList";

const parts: Task[] = [];
const asked: string[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { id?: string }) => {
    if (cmd === "parts_of") asked.push(args?.id ?? "");
    return Promise.resolve(cmd === "parts_of" ? parts : []);
  },
}));

const task = (id: string, title: string, more: Partial<Task> = {}): Task => ({
  id,
  title,
  status: "open",
  priority: "unset",
  order: id,
  tags: [],
  steps: [],
  log: [],
  reminders: [],
  ...more,
});

const day = (at: string): DateSpec => ({
  at: `${at}T09:00:00`,
  tz: "America/Santiago",
  floating: true,
  has_time: false,
});
const a3 = task("A3", "A3 · Migración de Sodimac Uruguay", { deadline: day("2026-12-04") });
const tenParts = Array.from({ length: 10 }, (_, at) =>
  task(`A3-${at}`, `Sodimac paso ${at + 1}`, { part_of: "A3" }),
);
const counting = (open: number, closed = 0): Record<string, Whole> => ({
  A3: { title: a3.title, open, closed },
});

beforeEach(() => {
  parts.splice(0, parts.length, ...tenParts);
  asked.splice(0, asked.length);
});

const listed = (tasks: Task[], wholes = counting(10), more = {}) =>
  render(
    <TaskList
      tasks={tasks}
      lists={[]}
      wholes={wholes}
      title="Hoy"
      onSelect={vi.fn()}
      onComplete={vi.fn()}
      {...more}
    />,
  );

const counter = (closed: number, all: number) =>
  screen.getByRole("button", {
    name: new RegExp(fill("partsClosed", String(closed), String(all))),
  });
const rows = () => screen.getAllByRole("listitem");

describe("a whole whose parts the view leaves out, as A3 in «Hoy»", () => {
  it("starts folded and asks for nothing until it is opened", () => {
    listed([a3]);

    expect(counter(0, 10).getAttribute("aria-expanded")).toBe("false");
    expect(rows()).toHaveLength(1);
    expect(asked).toEqual([]);
  });

  it("opens to all ten parts and folds them again", async () => {
    listed([a3]);

    fireEvent.click(counter(0, 10));
    await waitFor(() => expect(rows()).toHaveLength(11));
    expect(asked).toEqual(["A3"]);
    expect(counter(0, 10).getAttribute("aria-expanded")).toBe("true");

    fireEvent.click(counter(0, 10));
    expect(rows()).toHaveLength(1);
  });

  it("shows each part with its day, and only the open ones", async () => {
    parts.splice(
      0,
      parts.length,
      task("A3-0", "Inventario", { part_of: "A3", date: day("2026-11-02") }),
      task("A3-1", "Corte", { part_of: "A3", status: "done" }),
    );
    listed([a3], counting(1, 1));

    fireEvent.click(counter(1, 2));

    await waitFor(() => expect(screen.getByText("Inventario")).not.toBeNull());
    expect(screen.getByText(whenLabel(day("2026-11-02")))).not.toBeNull();
    expect(screen.queryByText("Corte")).toBeNull();
  });

  it("keeps the fold when the view comes back with fresh tasks", async () => {
    const { rerender } = listed([a3]);
    fireEvent.click(counter(0, 10));
    await waitFor(() => expect(rows()).toHaveLength(11));

    rerender(
      <TaskList
        tasks={[{ ...a3 }]}
        lists={[]}
        wholes={counting(10)}
        title="Hoy"
        onSelect={vi.fn()}
      />,
    );

    expect(rows()).toHaveLength(11);
  });

  it("hands back the part itself, since the view does not hold it", async () => {
    const selected = vi.fn();
    const completed = vi.fn();
    listed([a3], counting(10), { onSelect: selected, onComplete: completed });
    fireEvent.click(counter(0, 10));
    const first = await screen.findByText("Sodimac paso 1");

    fireEvent.click(first);
    fireEvent.click(screen.getByTitle(fill("completeIt", "Sodimac paso 1")));

    expect(selected).toHaveBeenCalledWith("A3-0", tenParts[0]);
    expect(completed).toHaveBeenCalledWith("A3-0", "Sodimac paso 1");
  });

  it("offers no fold when every part is closed", () => {
    listed([a3], counting(0, 2));

    expect(screen.queryByRole("button", { name: /▣/ })).toBeNull();
    expect(screen.getByText(/▣ 2\/2/)).not.toBeNull();
  });

  it("folds nothing in a dense listing", () => {
    listed([a3], counting(10), { dense: true });

    expect(screen.queryByRole("button", { expanded: false })).toBeNull();
  });
});

describe("a whole with parts both in and out of the view", () => {
  const inView = task("A3-0", "Sodimac paso 1", { part_of: "A3", date: day("2026-10-27") });
  const beyond = () => screen.getByRole("button", { name: new RegExp(fill("partsBeyond", "9")) });

  it("shows the ones in view and keeps the rest behind a row of their own", async () => {
    listed([a3, inView]);

    expect(rows()).toHaveLength(2);
    expect(beyond().getAttribute("aria-expanded")).toBe("false");

    fireEvent.click(beyond());

    await waitFor(() => expect(rows()).toHaveLength(11));
    expect(screen.getAllByText("Sodimac paso 1")).toHaveLength(1);
  });

  it("opens and folds from the keyboard, one level at a time", async () => {
    listed([a3, inView]);
    const whole = () => rows()[0] as HTMLElement;

    expect(whole().getAttribute("aria-keyshortcuts")).toContain("ArrowRight ArrowLeft");
    fireEvent.keyDown(whole(), { key: "ArrowRight" });
    await waitFor(() => expect(rows()).toHaveLength(11));

    fireEvent.keyDown(whole(), { key: "ArrowLeft" });
    expect(rows()).toHaveLength(1);

    fireEvent.keyDown(whole(), { key: "ArrowRight" });
    expect(rows()).toHaveLength(11);
  });

  it("opens a folded whole from the keyboard", async () => {
    listed([a3]);

    fireEvent.keyDown(rows()[0] as HTMLElement, { key: "ArrowRight" });

    await waitFor(() => expect(rows()).toHaveLength(11));
  });
});
