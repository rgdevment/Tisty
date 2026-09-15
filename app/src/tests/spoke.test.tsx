import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { Lozenge, Pip, spokenLabel } from "../ui/Spoke";
import { knowAgents } from "../who";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (extra: Partial<Task> = {}): Task =>
  ({
    id: "01T",
    title: "pasar biome sobre el front",
    status: "open",
    priority: "unset",
    order: "a0",
    ...extra,
  }) as Task;

const marked = { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1" };

describe("what the mark says out loud", () => {
  afterEach(() => knowAgents({}));

  it("is just the title when nobody spoke for it", () => {
    expect(spokenLabel(task())).toBe("pasar biome sobre el front");
  });

  it("names the agent when this machine knows it", () => {
    knowAgents({ dev_agent: "peral 76" });

    expect(spokenLabel(task({ resolved: marked }))).toBe(
      "pasar biome sobre el front — peral 76 says this is done",
    );
  });

  it("falls back to an unnamed agent when it does not", () => {
    expect(spokenLabel(task({ resolved: marked }))).toBe(
      "pasar biome sobre el front — An agent says this is done",
    );
  });

  it("speaks of a finished task in the past, and says it is finished", () => {
    expect(spokenLabel(task({ status: "done", resolved: marked }))).toBe(
      "pasar biome sobre el front — Done — an agent had said it was done",
    );
  });

  it("says a task was dropped even with no mark on it", () => {
    expect(spokenLabel(task({ status: "dropped" }))).toBe("pasar biome sobre el front — Dropped");
  });
});

describe("the pip inside the ring", () => {
  afterEach(() => knowAgents({}));

  it("is not drawn when nobody spoke for the task", () => {
    const { container } = render(<Pip task={task()} />);

    expect(container.firstChild).toBeNull();
  });

  it("carries the hour it was said, so hovering explains the colour", () => {
    render(<Pip task={task({ resolved: marked })} />);

    const pip = screen.getByTitle(/^An agent said this was done — .+/);
    expect(pip.className).toContain("bg-hue-teal");
  });
});

describe("the lozenge beside the hour", () => {
  afterEach(() => knowAgents({}));

  it("is not drawn when nobody spoke for the task", () => {
    const { container } = render(<Lozenge task={task()} />);

    expect(container.firstChild).toBeNull();
  });

  it("says who spoke, so the colour is never the only teller", () => {
    knowAgents({ dev_agent: "peral 76" });
    render(<Lozenge task={task({ resolved: marked })} />);

    const mark = screen.getByTitle("peral 76 says this is done");
    expect(mark.textContent).toBe("◆");
  });

  it("stays on a task already finished, and speaks of it in the past", () => {
    render(<Lozenge task={task({ status: "done", resolved: marked })} />);

    expect(screen.getByTitle("an agent had said it was done")).toBeTruthy();
  });
});
