import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "task_story" ? { pages: [] } : null),
}));

const task: Task = {
  id: "01A",
  title: "Renew the passport",
  status: "open",
  priority: "unset",
  order: "a0",
  tags: [],
  steps: [],
  log: [],
  reminders: [],
};

const opened = () =>
  render(
    <Detail
      task={task}
      lists={[]}
      known={[]}
      onCollapse={vi.fn()}
      onPatch={vi.fn()}
      onComplete={vi.fn()}
      onDiscard={vi.fn()}
      onReopen={vi.fn()}
      onStillOpen={vi.fn()}
      onErase={vi.fn()}
      onFold={vi.fn()}
      onReadAs={vi.fn()}
      onOpenToAgents={vi.fn()}
      onClose={vi.fn()}
    />,
  );

describe("the template of an open task", () => {
  it("is filled in by default, and edited from the footer once asked for", () => {
    opened();

    const shape = screen.getByRole("button", { name: new RegExp(t("shapeEdit")) });
    expect(shape.getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(shape);

    const done = screen.getByRole("button", { name: new RegExp(t("shapeDone")) });
    expect(done.getAttribute("aria-pressed")).toBe("true");
  });
});
