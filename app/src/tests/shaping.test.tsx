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

const more = () =>
  fireEvent.mouseDown(screen.getByRole("button", { name: t("more") }), { detail: 1 });

describe("the structure of an open task", () => {
  it("is filled in by default, and edited only once asked for", () => {
    opened();

    more();
    fireEvent.click(screen.getByRole("menuitem", { name: new RegExp(t("shapeEdit")) }));

    more();
    expect(screen.getByRole("menuitem", { name: new RegExp(t("shapeDone")) })).toBeTruthy();
  });
});
