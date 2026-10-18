import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

const dialog = vi.hoisted(() => ({ sure: true, asked: 0 }));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => {
    dialog.asked += 1;
    return Promise.resolve(dialog.sure);
  },
}));

const written: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "unset",
  order: "a0",
  description: "the one for accounting",
  steps: [{ id: "01S", text: "collect the figures", done: false, order: "a0" }],
  log: [
    {
      id: "01E",
      at: "2026-08-10 09:00:00",
      tz: "America/Santiago",
      body: "spoke to accounting",
    },
  ],
  volume: { steps: 1, steps_done: 0, journal: 1, described: true },
};

function open(task: Task = written) {
  const on = {
    patch: vi.fn(),
    log: vi.fn(),
    discard: vi.fn(),
    reopen: vi.fn(),
  };
  render(
    <Detail
      task={task}
      lists={[]}
      known={[]}
      onCollapse={() => {}}
      onPatch={on.patch}
      onComplete={() => {}}
      onDiscard={on.discard}
      onReopen={on.reopen}
      onStillOpen={() => {}}
      onErase={() => {}}
      onFold={() => {}}
      onReadAs={() => {}}
      onOpenToAgents={() => {}}
      onClose={() => {}}
    />,
  );
  return on;
}

const box = (name: string) =>
  screen.getByRole("textbox", {
    name: new RegExp(name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
  }) as HTMLTextAreaElement | HTMLInputElement;

describe("escape discards, blur keeps", () => {
  it("puts the title back and writes nothing", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("Title"));
    await user.type(box("Title"), "something else{Escape}");

    expect(on.patch).not.toHaveBeenCalled();
    expect(box("Title").value).toBe("write the report");
  });
});

describe("what an edit is allowed to do", () => {
  it("refuses to empty the title, because a task without one cannot be found", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("Title"));
    await user.tab();

    expect(on.patch).not.toHaveBeenCalled();
    expect(box("Title").value).toBe("write the report");
  });

  it("says nothing when the text comes back unchanged", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.click(box("Title"));
    await user.tab();

    expect(on.patch).not.toHaveBeenCalled();
  });
});

describe("a settled task", () => {
  it("offers to reopen instead of to discard", () => {
    open({ ...written, status: "done" });
    expect(screen.getByRole("button", { name: /Reopen/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Not doing it/ })).toBeNull();
  });
});
