import { invoke } from "@tauri-apps/api/core";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve(null)) }));

beforeEach(() => {
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    if (cmd === "task_story") {
      return Promise.resolve({
        id: "01A",
        pages: [{ n: 1, at: "2026-08-01 09:00:00", chapter: "made" }],
      });
    }
    if (cmd === "task_refs") return Promise.resolve([]);
    return Promise.resolve(null);
  });
});

const one = (status: Task["status"]): Task =>
  ({
    id: "01A",
    title: "write the report",
    status,
    priority: "unset",
    order: "a0",
    description: "the one for accounting",
    steps: [],
    log: [],
    volume: {},
  }) as unknown as Task;

const show = (status: Task["status"], expanded: boolean) =>
  render(
    <Detail
      task={one(status)}
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
      onClose={() => {}}
    />,
  );

describe("the side block with the trail", () => {
  it("keeps one trail and one «what it left» for a settled task in the column", async () => {
    show("done", false);
    await waitFor(() => expect(screen.getAllByText("The trail").length).toBeGreaterThan(0));

    expect(screen.getAllByText("The trail")).toHaveLength(1);
    expect(screen.getAllByText("What it carries")).toHaveLength(1);
  });

  it("keeps one of each full-screen too, for a settled task", async () => {
    show("done", true);
    await waitFor(() => expect(screen.getAllByText("The trail").length).toBeGreaterThan(0));

    expect(screen.getAllByText("The trail")).toHaveLength(1);
    expect(screen.getAllByText("What it carries")).toHaveLength(1);
  });

  it("keeps the sealed notice for a settled task, in the column and full-screen", async () => {
    const said = /This trail is not edited/;
    const { unmount } = show("dropped", false);
    await waitFor(() => expect(screen.getAllByText(said)).toHaveLength(1));
    unmount();

    show("dropped", true);
    await waitFor(() => expect(screen.getAllByText(said)).toHaveLength(1));
  });

  it("does not put the sealed notice over an open task full-screen", async () => {
    show("open", true);
    await waitFor(() => expect(screen.getAllByText("The trail").length).toBeGreaterThan(0));

    expect(screen.queryAllByText(/This trail is not edited/)).toHaveLength(0);
  });

  it("leaves an open task in the column without a trail, as v1.7.0 did", async () => {
    show("open", false);
    await waitFor(() => expect(screen.getByText("Description")).toBeTruthy());

    expect(screen.queryAllByText("The trail")).toHaveLength(0);
    expect(screen.queryAllByText("What it carries")).toHaveLength(0);
  });
});
