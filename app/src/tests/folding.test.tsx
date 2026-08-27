import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const done = (id: string, title: string, at: string): Task =>
  ({
    id,
    title,
    status: "done",
    priority: "unset",
    order: "a0",
    completed_at: at,
  }) as Task;

const archive = () =>
  render(
    <TaskList
      tasks={[
        done("1", "ship the release", "2026-08-25T09:00:00Z"),
        done("2", "renew the certificate", "2026-07-18T09:00:00Z"),
      ]}
      lists={[]}
      title="Archive"
      bands="month"
      onSelect={() => {}}
    />,
  );

describe("the bands of the archive fold", () => {
  it("puts away a whole band and says how much it holds", async () => {
    const user = userEvent.setup();
    archive();

    const band = screen.getAllByRole("button", { expanded: true })[0];
    await user.click(band);

    expect(screen.queryByText("ship the release")).toBeNull();
    expect(screen.getByText("renew the certificate")).toBeTruthy();
    expect(screen.getAllByRole("button", { expanded: false })[0].textContent).toMatch(/1/);
  });

  it("brings it back", async () => {
    const user = userEvent.setup();
    archive();

    const band = screen.getAllByRole("button", { expanded: true })[0];
    await user.click(band);
    await user.click(screen.getAllByRole("button", { expanded: false })[0]);

    expect(screen.getByText("ship the release")).toBeTruthy();
  });
});
