import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve(null)) }));

beforeEach(() => {
  vi.mocked(invoke).mockImplementation(() => Promise.resolve(null));
});

const task: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "unset",
  order: "a0",
  description: "the one for accounting",
  steps: [{ id: "01S", text: "collect the figures", done: false, order: "a0" }],
} as unknown as Task;

const show = (expanded = false) => {
  const closed = vi.fn();
  render(
    <Detail
      task={task}
      lists={[]}
      known={[]}
      expanded={expanded}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={() => {}}
      onStep={() => {}}
      onMark={() => {}}
      onDropStep={() => {}}
      onLog={() => {}}
      onComplete={() => {}}
      onDiscard={() => {}}
      onReopen={() => {}}
      onErase={() => {}}
      onClose={closed}
    />,
  );
  return closed;
};

describe("getting back out of an open task", () => {
  it("offers a close button in the column", async () => {
    const closed = show();

    await userEvent.click(screen.getByRole("button", { name: /close the task/i }));

    expect(closed).toHaveBeenCalled();
  });

  it("closes on Escape", async () => {
    const closed = show();

    (document.querySelector("aside") as HTMLElement).focus();
    await userEvent.keyboard("{Escape}");

    expect(closed).toHaveBeenCalled();
  });

  it("closes on Escape full-screen too", async () => {
    const closed = show(true);

    (document.querySelector("main") as HTMLElement).focus();
    await userEvent.keyboard("{Escape}");

    expect(closed).toHaveBeenCalled();
  });

  it("leaves Escape alone inside a field", async () => {
    const closed = show();

    await userEvent.click(screen.getByRole("textbox", { name: /title/i }));
    await userEvent.keyboard("{Escape}");

    expect(closed).not.toHaveBeenCalled();
  });

  it("shows where the focus went", () => {
    show();

    const panel = document.querySelector("aside") as HTMLElement;
    expect(document.activeElement).toBe(panel);
    expect(panel.className).toContain("focus-visible:ring-2");
  });
});

describe("what only the mouse used to see", () => {
  it("draws the remove button of a step once it has focus", () => {
    show();

    const remove = screen.getByRole("button", { name: /remove collect the figures/i });
    expect(remove.className).toContain("focus-visible:opacity-100");
  });

  it("rings the description when the keyboard reaches it", () => {
    show();

    const prose = screen.getByLabelText(/description/i);
    expect(prose.getAttribute("tabindex")).toBe("0");
    expect(prose.className).toContain("focus-visible:ring-2");
  });
});

describe("what an open task carries, once it takes the whole page", () => {
  it("counts what it has gathered beside the trail, from the day it was made", async () => {
    // Five days back from now, not a date written down: one of those is right today only.
    const made = new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString();
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      Promise.resolve(
        cmd === "task_story" ? { pages: [{ n: 1, at: made, chapter: "made" }] } : null,
      ),
    );
    show(true);

    expect(await screen.findByText("5")).toBeTruthy();
    expect(screen.getByText("days open")).toBeTruthy();
    expect(screen.getByText("0/0")).toBeTruthy();
    expect(screen.getByText("What it carries")).toBeTruthy();
  });

  it("leaves the trail out of the panel, where the column has no room for it", () => {
    show(false);

    expect(screen.queryByText("What it carries")).toBeNull();
  });
});
