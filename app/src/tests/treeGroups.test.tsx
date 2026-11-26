import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_ICON, AGENTS_NAME } from "../agentsSync";
import type { Filed, Folded, Papers, Wired } from "../core";
import { showAll } from "../hiding";
import { adopt } from "../locales";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  rows: [] as unknown[],
  skillsOn: true,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "places_rules") {
      const open = { makes: ["doc"], pages: true, moves: true, copies: true };
      const shut = { makes: [], pages: false, moves: false, copies: false };
      return Promise.resolve({ plain: open, agents: open, ideas: shut, diary: shut });
    }
    if (cmd === "agents_rows") return Promise.resolve(ipc.rows);
    if (cmd === "agents_state") {
      return Promise.resolve({
        folder: "A",
        held: 1,
        consented: true,
        allOn: true,
        assistants: [],
      });
    }
    if (cmd === "agents_tisty_skills") {
      return Promise.resolve({
        version: "1.50.0",
        on: ipc.skillsOn,
        leftover: [],
        skills: [],
        landing: [],
        plugged: [],
      });
    }
    return Promise.resolve([]);
  },
}));

const folder = (
  id: string,
  name: string,
  parent: string | null,
  more: Partial<Folded> = {},
): Folded => ({
  id,
  name,
  parent,
  icon: null,
  color: null,
  holds: 0,
  archived: false,
  away: false,
  ...more,
});

const doc = (id: string, title: string, at: string): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: at,
  archived: false,
  away: false,
});

const FOLDERS = [
  folder("A", AGENTS_NAME, null, { icon: AGENTS_ICON }),
  folder("G", "Global", "A"),
  folder("C", "Claude", "A", { holds: 1 }),
  folder("M", "desk", "A", { agents: { computer: "Desk", mine: true, room: false } }),
  folder("MC", "Claude", "M", { agents: { mine: true, room: true } }),
];
const RULE = doc("r1", "Horario", "C");
const PAPERS: Papers = { folders: FOLDERS, docs: [RULE] };

const ruleRow = {
  file: RULE.file,
  everyone: true,
  kind: "rule",
  level: "claude",
  name: "rules/horario.md",
  label: "Horario",
  skill: null,
};

const hooked = (more: Partial<Wired> = {}): Wired => ({
  id: "claude",
  name: "Claude",
  at: "/h/claude",
  wired: true,
  astray: false,
  maybe: false,
  byHand: false,
  manual: { file: "c.json", snippet: "{}" },
  hooks: { on: true, at: "/h/claude/hooks.json" },
  ...more,
});

beforeEach(() => {
  adopt("es");
  showAll();
  document.elementFromPoint = () => null;
  ipc.rows = [ruleRow];
  ipc.skillsOn = true;
});

const drawn = async (props: Partial<Parameters<typeof Tree>[0]> = {}) => {
  const view = render(<Tree papers={PAPERS} onOpen={vi.fn()} onFile={vi.fn()} {...props} />);
  for (let round = 0; round < 8; round += 1) {
    await waitFor(() => expect(screen.queryAllByRole("button").length).toBeGreaterThan(0));
    const shut = screen.queryAllByRole("button", {
      name: /^(Desplegar|Abrir|Open|Unfold|Expand) /,
    });
    if (shut.length === 0) break;
    for (const one of shut) fireEvent.click(one);
  }
  return view;
};

const asked = "Nueva regla de Claude, para todos tus equipos";

describe("the three groups of Global in the tree", () => {
  it("draws a rule for all under Global > Reglas > Claude, once, and not the folder it is kept in", async () => {
    const { container } = await drawn();
    await waitFor(() => expect(screen.getByText("Horario")).toBeTruthy());
    expect(screen.getAllByText("Horario")).toHaveLength(1);
    expect(screen.getByText("Reglas")).toBeTruthy();
    // The sub-heading of Global and the Claude folder inside this computer's; the root one is not drawn.
    expect(screen.getAllByText("Claude")).toHaveLength(2);
    const global = screen.getByText("Global");
    const heading = screen.getByText("Reglas");
    const rule = screen.getByText("Horario");
    const follows = (a: Node, b: Node) =>
      Boolean(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING);
    expect(follows(global, heading)).toBe(true);
    expect(follows(heading, rule)).toBe(true);
    expect(container.querySelector('[data-row="C"]')).toBeNull();
  });

  it("makes no heading a row nor a drop target, and the projected rule no drop target", async () => {
    const { container } = await drawn();
    await waitFor(() => expect(screen.getByText("Horario")).toBeTruthy());
    for (const word of ["Reglas", "Skills"]) {
      const heading = screen.getByText(word);
      expect(heading.closest("[data-row]")).toBeNull();
      expect(heading.closest("[data-drop]")).toBeNull();
    }
    expect(container.querySelector('[data-drop="r1"]')).toBeNull();
    expect(container.querySelector('[data-row="r1"]')).not.toBeNull();
  });

  it("calls onNewRule with the agent from the plus button, a row and not a folder", async () => {
    const onNewRule = vi.fn();
    const { container } = await drawn({ onNewRule });
    const plus = await screen.findByRole("button", { name: asked });
    expect(plus.getAttribute("data-row")).toBe("rules:claude");
    expect(plus.closest("[data-drop]")).toBeNull();
    expect(container.querySelector('[data-drop="rules:claude"]')).toBeNull();
    fireEvent.click(plus);
    expect(onNewRule).toHaveBeenCalledWith("claude");
    expect(onNewRule).toHaveBeenCalledTimes(1);
  });

  it("reaches the plus button with ArrowDown from the row above", async () => {
    const { container } = await drawn({ onNewRule: vi.fn() });
    const plus = await screen.findByRole("button", { name: asked });
    const global = container.querySelector<HTMLElement>('[data-row="G"]') as HTMLElement;
    global.focus();
    fireEvent.keyDown(global, { key: "ArrowDown" });
    expect(document.activeElement).toBe(plus);
    fireEvent.keyDown(plus, { key: "ArrowUp" });
    expect(document.activeElement).toBe(global);
  });

  it("speaks English too", async () => {
    adopt("en");
    await drawn({ onNewRule: vi.fn() });
    await screen.findByRole("button", { name: "New rule for Claude, for all your computers" });
    expect(screen.getByText("Rules")).toBeTruthy();
  });

  it("draws the Hooks heading only when an agent is connected with hooks, and its line opens them", async () => {
    const onHooks = vi.fn();
    const { rerender } = await drawn({ agents: [hooked({ hooks: null })], onHooks });
    await screen.findByText("Horario");
    expect(screen.queryByText("Hooks")).toBeNull();
    rerender(
      <Tree
        papers={PAPERS}
        onOpen={vi.fn()}
        onFile={vi.fn()}
        agents={[hooked()]}
        onHooks={onHooks}
      />,
    );
    const heading = await screen.findByText("Hooks");
    expect(heading.closest("[data-row]")).toBeNull();
    const line = screen.getByRole("button", { name: /Claude en este equipo/ });
    expect(line.getAttribute("data-row")).toBe("hooks:claude");
    fireEvent.click(line);
    expect(onHooks).toHaveBeenCalledTimes(1);
  });

  it("puts the Tisty skills row under Skills", async () => {
    await drawn();
    const skills = await screen.findByText("Skills");
    const row = await screen.findByText("Skills de Tisty");
    expect(skills.compareDocumentPosition(row) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(skills.closest("[data-row]")).toBeNull();
  });

  it("walks the Tisty skills row with the arrow keys, between the rules and the Hooks line", async () => {
    const { container } = await drawn({
      onNewRule: vi.fn(),
      agents: [hooked()],
      onHooks: vi.fn(),
    });
    const rule = (await waitFor(() => {
      const row = container.querySelector<HTMLElement>('[data-row="r1"]');
      expect(row).not.toBeNull();
      return row;
    })) as HTMLElement;
    const skills = (await waitFor(() => {
      const row = container.querySelector<HTMLElement>('[data-row="tisty-skills"]');
      expect(row).not.toBeNull();
      return row;
    })) as HTMLElement;
    const hooks = (await waitFor(() => {
      const row = container.querySelector<HTMLElement>('[data-row="hooks:claude"]');
      expect(row).not.toBeNull();
      return row;
    })) as HTMLElement;
    expect(["-1", "0"]).toContain(skills.getAttribute("tabindex"));
    rule.focus();
    fireEvent.keyDown(rule, { key: "ArrowDown" });
    expect(document.activeElement).toBe(skills);
    fireEvent.keyDown(skills, { key: "ArrowDown" });
    expect(document.activeElement).toBe(hooks);
    fireEvent.keyDown(hooks, { key: "ArrowUp" });
    expect(document.activeElement).toBe(skills);
    fireEvent.keyDown(skills, { key: "ArrowUp" });
    expect(document.activeElement).toBe(rule);
  });
});
