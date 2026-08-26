import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve({ id: "01A", pages: [] }),
}));

const closed: Task = {
  id: "01A",
  title: "renew the certificate",
  status: "done",
  priority: "do",
  order: "a0",
  description: "the authority took nine days to issue it",
  list: "01W",
  tags: ["release"],
  completed_at: "2026-08-12 10:00:00",
  steps: [{ id: "01S", text: "gather the papers", done: true, order: "a0" }],
  log: [{ id: "01J", at: "2026-08-10T09:00:00Z", body: "the form was the slow part" }],
  volume: { steps: 1, steps_done: 1, journal: 1, described: true },
};

const nothing = () => {};

const shown = (task: Task) =>
  render(
    <Detail
      task={task}
      lists={[{ id: "01W", name: "Work", order: "a0" }]}
      known={["release"]}
      expanded={false}
      onExpand={nothing}
      onCollapse={nothing}
      onPatch={nothing}
      onStep={nothing}
      onMark={nothing}
      onDropStep={nothing}
      onLog={nothing}
      onComplete={nothing}
      onDiscard={nothing}
      onReopen={nothing}
      onErase={nothing}
      onClose={nothing}
    />,
  );

describe("a closed task is sealed", () => {
  it("offers no way to rewrite what happened", () => {
    shown(closed);

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.queryByRole("checkbox")).toBeNull();
  });

  it("still shows everything it carried, only as a record", () => {
    shown(closed);

    expect(screen.getByText("renew the certificate")).toBeTruthy();
    expect(screen.getByText(/the authority took nine days/)).toBeTruthy();
    expect(screen.getByText("gather the papers")).toBeTruthy();
    expect(screen.getByText(/the form was the slow part/)).toBeTruthy();
    expect(screen.getByText("▤ Work")).toBeTruthy();
    expect(screen.getByText("◈ release")).toBeTruthy();
  });

  it("leaves reopening as the way back, and says what reopening does", () => {
    shown(closed);

    expect(screen.getByRole("button", { name: /Reopen/i })).toBeTruthy();
    expect(screen.getByText(/not edited/i)).toBeTruthy();
  });

  it("keeps an open task editable, which is the whole point of the difference", () => {
    shown({ ...closed, status: "open", completed_at: undefined });

    expect(screen.getByRole("textbox", { name: "Title" })).toBeTruthy();
  });
});
