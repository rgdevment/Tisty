import { act, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentRow } from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
import { grouped, inSkill, useAgentRows } from "../ui/agentRows";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  rows: 0,
  only: [] as (string[] | null)[],
  answer: (): Promise<unknown> => Promise.resolve([]),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { only?: string[] | null }) => {
    if (cmd !== "agents_rows") return Promise.resolve(null);
    ipc.rows += 1;
    ipc.only.push(args?.only ?? null);
    return ipc.answer();
  },
}));

const folder = (
  id: string,
  name: string,
  parent: string | null,
  icon: string | null = null,
  holds = 0,
) => ({ id, name, parent, icon, holds, archived: false, away: false }) as Folded;

const doc = (file: string, title: string, folderId: string | null = "claude"): Filed => ({
  id: `id-${file}`,
  file,
  title,
  folder: folderId,
  archived: false,
  away: false,
  wrote: "2026-11-19T10:00:00Z",
});

const row = (file: string, name: string, label: string, kind: AgentRow["kind"] = "rule") => {
  const skill = kind === "skill" && name.includes("/") ? name.split("/")[0] : null;
  return { file, kind, level: "claude", name, label, skill } satisfies AgentRow;
};

const FOLDERS = [
  folder("agents", "AGENTS", null, "robot"),
  folder("claude", "Claude", "agents", null, 5),
];

const ROWS: AgentRow[] = [
  row("d-1", "rules/horario.md", "Horario laboral"),
  row("d-2", "pr/references/body.md", "How the body reads", "skill"),
  row("d-3", "pr/SKILL.md", "pr", "skill"),
  row("d-4", "MEMORY.md", "MEMORY.md", "memory"),
  row("d-5", "solo/SKILL.md", "solo", "skill"),
];

const DOCS = [
  doc("d-1", "claude · rules/horario.md"),
  doc("d-2", "claude · pr/references/body.md"),
  doc("d-3", "claude · pr/SKILL.md"),
  doc("d-4", "claude · MEMORY.md"),
  doc("d-5", "claude · solo/SKILL.md"),
];

const PAPERS: Papers = { folders: FOLDERS, docs: DOCS };

const byFile = new Map(ROWS.map((one) => [one.file, one]));

beforeEach(() => {
  ipc.rows = 0;
  ipc.only = [];
  ipc.answer = () => Promise.resolve(ROWS);
});

describe("grouping the documents of AGENTS", () => {
  it("folds the files of a skill where the first stands, its SKILL.md first and named by it", () => {
    const drawn = grouped(DOCS, byFile);
    expect(drawn.map((one) => ("doc" in one ? one.doc.file : one.skill.label))).toEqual([
      "d-1",
      "pr",
      "d-4",
      "d-5",
    ]);
    const skill = drawn[1];
    if (!("skill" in skill)) throw new Error("a skill row");
    expect(skill.skill.docs.map((one) => one.file)).toEqual(["d-3", "d-2"]);
  });

  it("names a skill by its folder until its SKILL.md is known, and leaves other documents alone", () => {
    const plain = doc("x-1", "Compras", null);
    const drawn = grouped(
      [DOCS[1], plain, { ...DOCS[1], file: "d-6", id: "d-6" }],
      new Map([
        ["d-2", ROWS[1]],
        ["d-6", row("d-6", "pr/notes.md", "Notes", "skill")],
      ]),
    );
    expect(drawn).toHaveLength(2);
    expect("skill" in drawn[0] && drawn[0].skill.label).toBe("pr");
    expect("doc" in drawn[1] && drawn[1].doc.title).toBe("Compras");
  });

  it("calls a file inside its skill by its path there", () => {
    expect(inSkill(ROWS[1])).toBe("references/body.md");
    expect(inSkill(ROWS[0])).toBe("rules/horario.md");
  });
});

describe("asking for the rows", () => {
  it("never asks without AGENTS", () => {
    const { result } = renderHook(() =>
      useAgentRows({ folders: [folder("f", "Trabajo", null)], docs: [doc("a", "A", "f")] }),
    );
    expect(result.current.size).toBe(0);
    expect(ipc.rows).toBe(0);
  });

  it("asks again only when a document under AGENTS changes, and keeps the rows a failed ask leaves", async () => {
    const { result, rerender } = renderHook(({ papers }) => useAgentRows(papers), {
      initialProps: { papers: PAPERS },
    });
    await waitFor(() => expect(result.current.get("d-1")?.label).toBe("Horario laboral"));
    rerender({ papers: { ...PAPERS } });
    expect(ipc.rows).toBe(1);
    expect(ipc.only[0]).toEqual(["d-1", "d-2", "d-3", "d-4", "d-5"]);
    ipc.answer = () => Promise.reject(new Error("gone"));
    rerender({ papers: { ...PAPERS, docs: [...DOCS, doc("d-7", "x")] } });
    await act(async () => {});
    expect(ipc.rows).toBe(2);
    expect(ipc.only[1]).toEqual(["d-7"]);
    expect(result.current.get("d-1")?.label).toBe("Horario laboral");
  });

  it("forgets a document that left AGENTS at once, and one whose card is gone once asked", async () => {
    const { result, rerender } = renderHook(({ papers }) => useAgentRows(papers), {
      initialProps: { papers: PAPERS },
    });
    await waitFor(() => expect(result.current.size).toBe(5));
    const moved = DOCS.map((one) => (one.file === "d-1" ? { ...one, folder: null } : one));
    rerender({ papers: { ...PAPERS, docs: moved } });
    expect(result.current.has("d-1")).toBe(false);
    ipc.answer = () => Promise.resolve([]);
    const rewritten = moved.map((one) =>
      one.file === "d-4" ? { ...one, wrote: "2026-11-19T11:00:00Z" } : one,
    );
    rerender({ papers: { ...PAPERS, docs: rewritten } });
    await waitFor(() => expect(result.current.has("d-4")).toBe(false));
    expect(ipc.only[ipc.only.length - 1]).toEqual(["d-4"]);
    expect(result.current.get("d-2")?.label).toBe("How the body reads");
  });

  it("keeps only the answer for the documents now shown", async () => {
    const slow: { give?: (rows: AgentRow[]) => void } = {};
    ipc.answer = () => new Promise((give) => (slow.give = give));
    const { result, rerender } = renderHook(({ papers }) => useAgentRows(papers), {
      initialProps: { papers: PAPERS },
    });
    const first = slow.give;
    ipc.answer = () => Promise.resolve([row("d-1", "rules/horario.md", "Nuevo")]);
    rerender({ papers: { ...PAPERS, docs: [...DOCS, doc("d-7", "x")] } });
    await waitFor(() => expect(result.current.get("d-1")?.label).toBe("Nuevo"));
    expect(ipc.only[1]).toContain("d-1");
    await act(async () => first?.(ROWS));
    expect(result.current.get("d-1")?.label).toBe("Nuevo");
  });
});

const skillRow = () =>
  document.querySelector<HTMLButtonElement>("[data-skill]") as HTMLButtonElement;

describe("AGENTS in the sidebar", () => {
  const opened = async (open?: string) => {
    render(<Tree papers={PAPERS} open={open} onOpen={() => {}} onFile={() => {}} unfolded />);
    await screen.findByRole("button", { name: "Horario laboral" });
  };

  it("marks an assistant's folder with its logo", async () => {
    await opened();
    expect(document.querySelector('[data-mark="claude"]')).toBeTruthy();
  });

  it("shows each document by its name with its path on hover", async () => {
    await opened();
    expect(screen.getByRole("button", { name: "Horario laboral" }).title).toBe(
      "claude · rules/horario.md",
    );
    expect(screen.queryByText("claude · rules/horario.md")).toBeNull();
    expect(screen.getByRole("button", { name: "solo" })).toBeTruthy();
  });

  it("draws a skill as one row that says how many files it holds and opens to them", async () => {
    render(<Tree papers={PAPERS} onOpen={() => {}} onFile={() => {}} />);
    for (const one of screen.queryAllByRole("button", { name: /^Open / })) fireEvent.click(one);
    fireEvent.click(await screen.findByRole("button", { name: "Open Claude" }));
    await screen.findByRole("button", { name: "pr, 2 files" });
    const skill = skillRow();
    expect(skill.textContent).toContain("2 files");
    expect(skill.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByRole("button", { name: "references/body.md" })).toBeNull();
    fireEvent.click(skill);
    expect(screen.getByRole("button", { name: "references/body.md" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "SKILL.md" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "SKILL.md" }).closest("[data-drop]")).toBeNull();
    expect(screen.getByRole("button", { name: "MEMORY.md" }).closest("[data-drop]")).toBeTruthy();
    expect(skill.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByRole("button", { name: "Hide the files of pr" })).toBeTruthy();
    fireEvent.keyDown(skill, { key: "ArrowLeft" });
    expect(screen.queryByRole("button", { name: "SKILL.md" })).toBeNull();
    fireEvent.keyDown(skill, { key: "ArrowRight" });
    expect(screen.getByRole("button", { name: "SKILL.md" })).toBeTruthy();
  });

  it("opens the skill holding the open document by itself", async () => {
    render(<Tree papers={PAPERS} open="d-2" onOpen={() => {}} onFile={() => {}} unfolded />);
    expect(await screen.findByRole("button", { name: "references/body.md" })).toBeTruthy();
    expect(skillRow().getAttribute("aria-expanded")).toBe("true");
  });

  it("moves between rows with the arrows from a skill row", async () => {
    await opened();
    const skill = skillRow();
    skill.focus();
    fireEvent.keyDown(skill, { key: "ArrowDown" });
    expect(document.activeElement?.getAttribute("aria-label")).toBe("SKILL.md");
    fireEvent.keyDown(skill, { key: "ArrowUp" });
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Horario laboral");
    fireEvent.keyDown(skill, { key: "Enter" });
  });
});
