import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { fill, t } from "../locales";
import AgentWord from "../ui/AgentWord";
import { knowAgents } from "../who";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (said: Record<string, unknown> | undefined, body?: string): Task =>
  ({
    id: "1",
    title: "pasar biome",
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    log: body === undefined ? [] : [{ id: "e1", body }],
    resolved: said && { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1", ...said },
  }) as unknown as Task;

const show = (one: Task, signing: "named" | "agent" | "mark" = "agent") =>
  render(<AgentWord task={one} signing={signing} onConfirm={vi.fn()} onKeepOpen={vi.fn()} />);

const proof = (container: HTMLElement) => container.querySelector<HTMLElement>("[data-proof]");

afterEach(() => knowAgents({}, { clients: {} }));

describe("the proof shown on the confirm row", () => {
  it("shows the first two lines of what the agent logged, without markdown marks", () => {
    const { container } = show(task({}, "## Done\n- **12** tests `passed`\nthird line"));

    const lines = Array.from(proof(container)?.querySelectorAll("p") ?? []).map(
      (p) => p.textContent,
    );

    expect(lines).toEqual(["Done", "12 tests passed"]);
    expect(screen.queryByText("third line")).toBeNull();
  });

  it("shows nothing when the log has no such entry", () => {
    const { container } = show(task({ entry: "zz" }, "ignored"));

    expect(proof(container)).toBeNull();
  });

  it("shows nothing when the entry is blank", () => {
    const { container } = show(task({}, "  \n\n"));

    expect(proof(container)).toBeNull();
  });

  it("shows the branch the work landed on", () => {
    const { container } = show(task({ branch: "feat/close-itself" }));

    expect(proof(container)?.textContent).toBe("⎇ feat/close-itself");
  });

  it("shows the pull request", () => {
    const { container } = show(task({ pr: "42" }));

    expect(proof(container)?.textContent).toBe(fill("saidPr", "42"));
  });

  it("shows branch and pull request together, after the proof lines", () => {
    const { container } = show(task({ branch: "fix/x", pr: "7" }, "all green"));

    const text = proof(container)?.textContent ?? "";

    expect(text).toBe(`all green⎇ fix/x · ${fill("saidPr", "7")}`);
  });

  it("keeps the confirm and keep-open buttons whether or not there is proof", () => {
    show(task({ branch: "fix/x" }, "all green"));

    expect(screen.getByRole("button", { name: new RegExp(t("markDone")) })).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("stillOpen")) })).toBeTruthy();
  });

  it("shows the proof for a task said not to be done too", () => {
    const { container } = show(task({ drop: true }, "not needed any more"));

    expect(proof(container)?.textContent).toBe("not needed any more");
    expect(screen.getByRole("button", { name: new RegExp(t("discardIt")) })).toBeTruthy();
  });

  it("shows nothing at all for a task nobody spoke about", () => {
    const { container } = show(task(undefined));

    expect(container.firstChild).toBeNull();
  });

  it("shows nothing for a task already closed", () => {
    const one = { ...task({}, "done"), status: "done" } as unknown as Task;
    const { container } = show(one);

    expect(container.firstChild).toBeNull();
  });
});

describe("the signing around the proof", () => {
  it.each([
    ["agent", t("agentSaidDone")],
    ["mark", t("saidMarkDone")],
  ] as const)("words the claim as %s signing", (signing, words) => {
    show(task({ via: "claude" }, "proof"), signing);

    expect(screen.getByText(words)).toBeTruthy();
    expect(screen.getByText("proof")).toBeTruthy();
  });

  it("names the client in a named signing", () => {
    knowAgents({}, { clients: { claude: "Claude Code" } });
    show(task({ via: "claude" }, "proof"), "named");

    expect(screen.getByText(fill("agentNamedSaidDone", "Claude Code"))).toBeTruthy();
  });

  it("falls back to an agent when a named signing finds no client", () => {
    show(task({}, "proof"), "named");

    expect(screen.getByText(t("agentSaidDone"))).toBeTruthy();
  });
});
