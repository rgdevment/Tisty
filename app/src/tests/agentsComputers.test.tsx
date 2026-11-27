import { act, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  AGENTS_CHANGED,
  AGENTS_ICON,
  AGENTS_NAME,
  type AgentsState,
  agentsDrawn,
  folderShown,
  ofAnother,
} from "../agentsSync";
import type { Filed, Folded, Papers } from "../core";
import { showAll } from "../hiding";
import { adopt } from "../locales";
import { useSkillsRow } from "../ui/AgentsMark";
import { FolderMark, levelOf } from "../ui/AssistantMark";
import Folder from "../ui/Folder";
import Glyph from "../ui/Glyph";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  state: null as unknown,
  skills: { on: false, leftover: [] as unknown[] },
  asked: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.asked.push(cmd);
    if (cmd === "agents_state") return Promise.resolve(ipc.state);
    if (cmd === "agents_tisty_skills") {
      return Promise.resolve({
        version: "1.50.0",
        skills: [],
        landing: [],
        plugged: [],
        ...ipc.skills,
      });
    }
    return Promise.resolve([]);
  },
}));

const stateWith = (assistants: AgentsState["assistants"]): AgentsState => ({
  folder: "A",
  held: 1,
  consented: true,
  allOn: true,
  assistants,
});

const CLAUDE_ON = [{ id: "claude", name: "Claude", on: true }];

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

const A = folder("A", AGENTS_NAME, null, { icon: AGENTS_ICON });
const GLOBAL = folder("G", "Global", "A", { holds: 1 });
const EMPTY_CLAUDE = folder("EC", "Claude", "A");
const EMPTY_CODEX = folder("EX", "Codex", "A");
const MINE = folder("M", "desk", "A", { agents: { computer: "Desk", mine: true, room: false } });
const MINE_CLAUDE = folder("MC", "Claude", "M", {
  holds: 2,
  agents: { mine: true, room: true },
});
const OTHER = folder("O", "laptop", "A", {
  agents: { computer: "Laptop", mine: false, room: false },
});
const OTHER_CLAUDE = folder("OC", "Claude", "O", {
  holds: 1,
  agents: { mine: false, room: true },
});

const FOLDERS = [A, GLOBAL, EMPTY_CLAUDE, EMPTY_CODEX, OTHER, OTHER_CLAUDE, MINE, MINE_CLAUDE];

const file = (id: string, title: string, at: string): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: at,
  archived: false,
  away: false,
});

const DOCS = [
  file("d1", "Tono", "G"),
  file("d2", "Mine rule", "MC"),
  file("d3", "Laptop rule", "OC"),
];

beforeEach(() => {
  adopt("en");
  showAll();
  ipc.state = stateWith(CLAUDE_ON);
  ipc.skills = { on: false, leftover: [] };
  ipc.asked = [];
});

describe("how a folder of a computer reads", () => {
  it("says this computer's folder with its name and 'this computer', another's with its name alone", () => {
    expect(folderShown(MINE)).toBe("Desk (this computer)");
    expect(folderShown(OTHER)).toBe("Laptop");
    expect(folderShown(MINE_CLAUDE)).toBe("Claude");
    expect(folderShown(folder("x", "Casa", null))).toBe("Casa");
  });

  it("says it in Spanish too", () => {
    adopt("es");
    expect(folderShown(MINE)).toBe("Desk (este equipo)");
    expect(folderShown(A)).toBe("Agentes");
  });

  it("tells another computer's folders from this one's and from the rest", () => {
    expect(ofAnother(OTHER)).toBe(true);
    expect(ofAnother(OTHER_CLAUDE)).toBe(true);
    expect(ofAnother(MINE)).toBe(false);
    expect(ofAnother(MINE_CLAUDE)).toBe(false);
    expect(ofAnother(GLOBAL)).toBe(false);
    expect(ofAnother(null)).toBe(false);
    expect(ofAnother(undefined)).toBe(false);
  });
});

describe("the folders of Agents as they are drawn", () => {
  const under = (parent: string) => FOLDERS.filter((one) => one.parent === parent);

  it("leaves out the assistants of the root with nothing in them, and puts other computers last", () => {
    const drawn = agentsDrawn(under("A"), FOLDERS, "A");
    expect(drawn.map((one) => one.id)).toEqual(["G", "M", "O"]);
  });

  it("keeps an empty assistant that has a folder inside, a document, an icon or a shelf", () => {
    const inside = [...FOLDERS, folder("sub", "rules", "EC")];
    expect(agentsDrawn(under("A"), inside, "A").map((one) => one.id)).toContain("EC");
    const holding = { ...EMPTY_CODEX, holds: 1 };
    expect(agentsDrawn([holding], FOLDERS, "A")).toEqual([holding]);
    const iconed = { ...EMPTY_CODEX, icon: "star" };
    expect(agentsDrawn([iconed], FOLDERS, "A")).toEqual([iconed]);
    const shelved = { ...EMPTY_CODEX, away: true };
    expect(agentsDrawn([shelved], FOLDERS, "A")).toEqual([shelved]);
  });

  it("keeps an empty folder of another name at the root, and an assistant's of a computer", () => {
    const own = folder("own", "Notas", "A");
    expect(agentsDrawn([own, folder("e", "Claude", "M")], FOLDERS, "A")).toHaveLength(2);
    const room = folder("r", "Codex", "A", { agents: { mine: true, room: true } });
    expect(agentsDrawn([room], FOLDERS, "A")).toEqual([room]);
  });

  it("changes nothing without Agents", () => {
    expect(agentsDrawn([EMPTY_CLAUDE, OTHER, MINE], FOLDERS, null)).toEqual([
      EMPTY_CLAUDE,
      OTHER,
      MINE,
    ]);
    expect(agentsDrawn([EMPTY_CLAUDE], FOLDERS, undefined)).toEqual([EMPTY_CLAUDE]);
  });
});

describe("the mark of a folder of a computer", () => {
  const glyphOf = (name: string) => {
    const { container, unmount } = render(<Glyph name={name} />);
    const said = container.querySelector("svg.glyph")?.innerHTML ?? "";
    unmount();
    return said;
  };

  it("gives an assistant's folder inside a computer's its mark, and none to the computer's own", () => {
    expect(levelOf(MINE_CLAUDE, "A")).toBe("claude");
    expect(levelOf(OTHER_CLAUDE, "A")).toBe("claude");
    expect(
      levelOf(
        folder("c", "Claude", "M", { agents: { computer: "Claude", mine: true, room: true } }),
        "A",
      ),
    ).toBeNull();
    expect(levelOf(MINE, "A")).toBeNull();
    expect(levelOf(MINE_CLAUDE, null)).toBeNull();
    expect(levelOf({ ...MINE_CLAUDE, icon: "star" }, "A")).toBeNull();
    expect(
      levelOf(folder("d", "Claude", "MC", { agents: { mine: true, room: false } }), "A"),
    ).toBeNull();
  });

  it("draws a computer's folder with the monitor unless it has an icon of its own", () => {
    const plain = render(<FolderMark folder={MINE} root="A" />);
    expect(plain.container.querySelector("svg.glyph")?.innerHTML).toBe(glyphOf("monitor"));
    plain.unmount();
    const iconed = render(<FolderMark folder={{ ...MINE, icon: "star" }} root="A" />);
    expect(iconed.container.querySelector("svg.glyph")?.innerHTML).toBe(glyphOf("star"));
    iconed.unmount();
    const room = render(<FolderMark folder={MINE_CLAUDE} root="A" />);
    expect(room.container.querySelector('[data-mark="claude"]')).toBeTruthy();
  });
});

describe("when Tisty's skills have a row", () => {
  it("is not while the agents are unknown", () => {
    const { result } = renderHook(() => useSkillsRow(null));
    expect(result.current).toBe(false);
    expect(ipc.asked).toEqual([]);
  });

  it("is when some agent is on this computer, without asking about the skills", () => {
    const { result } = renderHook(() => useSkillsRow(stateWith(CLAUDE_ON)));
    expect(result.current).toBe(true);
    expect(ipc.asked).toEqual([]);
  });

  it("is not with no agent and the skills off and nothing left over", async () => {
    const { result } = renderHook(() => useSkillsRow(stateWith([])));
    await waitFor(() => expect(ipc.asked).toContain("agents_tisty_skills"));
    expect(result.current).toBe(false);
  });

  it("is with no agent when the skills are on", async () => {
    ipc.skills = { on: true, leftover: [] };
    const { result } = renderHook(() => useSkillsRow(stateWith([])));
    await waitFor(() => expect(result.current).toBe(true));
  });

  it("is with no agent when copies are left over, and follows the next round", async () => {
    ipc.skills = { on: false, leftover: [{ assistant: "claude", at: "/x", changed: false }] };
    const { result } = renderHook(() => useSkillsRow(stateWith([])));
    await waitFor(() => expect(result.current).toBe(true));
    ipc.skills = { on: false, leftover: [] };
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    await waitFor(() => expect(result.current).toBe(false));
  });

  it("keeps the last answer when asking fails and goes quiet when an agent comes", async () => {
    const { result, rerender } = renderHook(({ at }) => useSkillsRow(at), {
      initialProps: { at: stateWith([]) },
    });
    await waitFor(() => expect(ipc.asked).toContain("agents_tisty_skills"));
    rerender({ at: stateWith(CLAUDE_ON) });
    expect(result.current).toBe(true);
    ipc.asked = [];
    act(() => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    expect(ipc.asked).toEqual([]);
  });
});

const PAPERS: Papers = { folders: FOLDERS, docs: DOCS };

const unfold = () => {
  for (let round = 0; round < 8; round += 1) {
    const shut = screen.queryAllByRole("button", { name: /^Open / });
    if (shut.length === 0) return;
    for (const one of shut) fireEvent.click(one);
  }
};

const drawnTree = async (papers: Papers = PAPERS) => {
  const view = render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} />);
  await waitFor(() => expect(ipc.asked).toContain("agents_state"));
  unfold();
  return view;
};

describe("the sidebar with computers in Agents", () => {
  it("draws each computer by its name, an assistant only inside them, and not an empty one at the root", async () => {
    await drawnTree();
    expect(screen.getByText("Desk (this computer)")).toBeTruthy();
    expect(screen.getByText("Laptop")).toBeTruthy();
    expect(screen.getAllByText("Claude")).toHaveLength(2);
    expect(screen.queryByText("Codex")).toBeNull();
    expect(screen.queryByText("desk")).toBeNull();
  });

  it("puts another computer's folder last and faint, with its mark dimmed", async () => {
    const { container } = await drawnTree();
    const mine = screen.getByText("Desk (this computer)");
    const theirs = screen.getByText("Laptop");
    expect(mine.compareDocumentPosition(theirs) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(theirs.closest("li")?.className).toContain("text-faint");
    expect(theirs.closest("button")?.className).toContain("text-faint");
    expect(mine.closest("li")?.className).not.toContain("text-faint");
    expect(mine.closest("button")?.className).not.toContain("text-faint");
    expect(container.querySelectorAll(".opacity-50").length).toBeGreaterThan(0);
  });

  it("does not call a folder empty because its only child is an empty assistant left out", async () => {
    const only: Papers = {
      folders: [A, EMPTY_CLAUDE, EMPTY_CODEX],
      docs: [],
    };
    await drawnTree(only);
    expect(screen.queryByText("Claude")).toBeNull();
    expect(screen.getByText("empty")).toBeTruthy();
  });

  it("has the Tisty's skills row under Global only where an agent is on this computer", async () => {
    const view = await drawnTree();
    await waitFor(() =>
      expect(view.container.querySelector("[data-row=tisty-skills]")).not.toBeNull(),
    );
    view.unmount();

    ipc.state = stateWith([]);
    const bare = await drawnTree();
    await waitFor(() => expect(ipc.asked).toContain("agents_tisty_skills"));
    expect(bare.container.querySelector("[data-row=tisty-skills]")).toBeNull();
  });
});

describe("the folder view with computers in Agents", () => {
  const view = (at: Folded = A, docs: Filed[] = DOCS) =>
    render(<Folder folder={at} folders={FOLDERS} docs={docs} onOpen={vi.fn()} onHere={vi.fn()} />);

  it("draws the computers by name, the empty assistants of the root not at all, and the others' last", async () => {
    view();
    const mine = await screen.findByRole("button", { name: /^Desk \(this computer\)/ });
    const theirs = screen.getByRole("button", { name: /^Laptop/ });
    expect(screen.queryByRole("button", { name: /^Codex/ })).toBeNull();
    expect(mine.compareDocumentPosition(theirs) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("makes another computer's folder faint and its mark dim", async () => {
    view();
    const mine = await screen.findByRole("button", { name: /^Desk \(this computer\)/ });
    const theirs = screen.getByRole("button", { name: /^Laptop/ });
    expect(theirs.querySelector(".truncate")?.className).toContain("text-faint");
    expect(theirs.querySelector(".opacity-50")).not.toBeNull();
    expect(mine.querySelector(".truncate")?.className).toContain("text-ink");
    expect(mine.querySelector(".opacity-50")).toBeNull();
  });

  it("shows the assistants inside a computer's own folder", async () => {
    view(MINE);
    expect(await screen.findByRole("button", { name: /^Claude/ })).toBeTruthy();
  });

  it("gives Global no skills row when no agent is on this computer", async () => {
    ipc.state = stateWith([]);
    const { container } = view(GLOBAL);
    await waitFor(() => expect(ipc.asked).toContain("agents_tisty_skills"));
    expect(container.querySelector("[data-row=tisty-skills]")).toBeNull();
    expect(screen.queryByText("empty")).toBeNull();
  });
});
