import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Menu from "../ui/Menu";
import Slash from "../ui/Slash";
import Steps from "../ui/Steps";
import TaskList from "../ui/TaskList";
import type { Step, Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const step = (done: boolean): Step => ({ id: "s1", text: "reunir las cifras", done, order: "a0" });

const task = (status: Task["status"]): Task => ({
  id: "01A",
  title: "una tarea",
  status,
  priority: 4,
  order: "a0",
  tags: [],
  steps: [],
  log: [],
  reminders: [],
});

const stepped = (done: boolean) =>
  render(<Steps steps={[step(done)]} onWrite={vi.fn()} onMark={vi.fn()} onDrop={vi.fn()} />);

describe("what a screen reader is told", () => {
  it("says whether a step is done, not just paints it", () => {
    stepped(false);

    expect(screen.getByRole("checkbox").getAttribute("aria-checked")).toBe("false");
  });

  it("flips that state when the step is done", () => {
    stepped(true);

    expect(screen.getByRole("checkbox").getAttribute("aria-checked")).toBe("true");
  });

  it("does not call the mark button and its field the same thing", () => {
    stepped(false);

    const mark = screen.getByRole("checkbox").getAttribute("aria-label");
    const field = screen.getByLabelText(/^Step:/).getAttribute("aria-label");

    expect(mark).not.toBe(field);
    expect(mark).toMatch(/reunir las cifras/);
    expect(field).toMatch(/reunir las cifras/);
  });

  it("says a task is done, not only with a glyph", () => {
    render(<TaskList tasks={[task("done")]} lists={[]} title="Hoy" onSelect={vi.fn()} />);

    expect(screen.getByRole("listitem").getAttribute("aria-label")).toMatch(/una tarea —/);
  });

  it("leaves an open task named by its title alone", () => {
    render(<TaskList tasks={[task("open")]} lists={[]} title="Hoy" onSelect={vi.fn()} />);

    expect(screen.getByRole("listitem").getAttribute("aria-label")).toBe("una tarea");
  });
});

describe("where the focus goes", () => {
  it("gives it back to whoever opened the menu", async () => {
    const opener = document.createElement("button");
    document.body.append(opener);
    opener.focus();

    const { unmount } = render(
      <Menu
        at={{ x: 10, y: 10 }}
        label="Más"
        choices={[{ key: "a", label: "Uno" }]}
        onClose={vi.fn()}
      />,
    );
    await waitFor(() => expect(document.activeElement).not.toBe(opener));

    unmount();

    await waitFor(() => expect(document.activeElement).toBe(opener));
    opener.remove();
  });
});

describe("a menu with nothing in it", () => {
  it("says so instead of showing an empty box", () => {
    render(<Slash at={{ x: 10, y: 10 }} blocks={[]} active={0} onPick={vi.fn()} />);

    expect(screen.getByText(/nothing matches/i)).toBeTruthy();
  });
});
