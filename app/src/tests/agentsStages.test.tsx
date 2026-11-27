import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  AGENTS_FIRST,
  AGENTS_SYNC,
  type AgentsState,
  agentsLit,
  agentsPaint,
  agentsStage,
  folderShown,
  isAgentsFolder,
  onlySwitchedOff,
} from "../agentsSync";
import { agentsEn, agentsEs } from "../agentsWords";
import type { Filed, Folded, Papers, Snapshot } from "../core";
import { deskOf } from "../desk";
import { AGENTS, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import { foundSaid } from "../ui/AgentsFound";
import Folder from "../ui/Folder";
import Modal from "../ui/Modal";
import SideRail from "../ui/SideRail";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  state: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "agents_state") return Promise.resolve(ipc.state);
    if (cmd === "agents_rows") return Promise.resolve([]);
    return Promise.resolve(null);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);
const settle = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
  ...more,
});

const filed = (id: string, at: string | null, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title: `Doc ${id}`,
  folder: at,
  archived: false,
  away: false,
  ...more,
});

const state = (more: Partial<AgentsState> = {}): AgentsState => ({
  folder: "A",
  held: 0,
  consented: false,
  allOn: true,
  assistants: [],
  ...more,
});

const root = folder("A", "AGENTS", { icon: "robot", color: "purple", holds: 3 });
const bare: Papers = { folders: [folder("W", "trabajo")], docs: [] };
const empty: Papers = { folders: [root, folder("W", "trabajo")], docs: [] };
const held: Papers = { folders: [root, folder("W", "trabajo")], docs: [filed("D1", "A")] };

const unbroughtName = "Agents — nothing brought yet";
const broughtName = "Agents — Not synced with your agents";

const tree = (papers: Papers, more: Partial<React.ComponentProps<typeof Tree>> = {}) =>
  render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} {...more} />);

const rail = (papers: Papers, onReveal = vi.fn()) =>
  render(
    <SideRail
      lists={[]}
      papers={papers}
      counts={{}}
      chosen={{ named: "tasks" }}
      onChoose={vi.fn()}
      onReveal={onReveal}
    />,
  );

const listening = (event: string) => {
  const heard = vi.fn();
  window.addEventListener(event, heard);
  return { heard, off: () => window.removeEventListener(event, heard) };
};

/// The mark drawn before the name: the Grip of a folder row, or the bare icon of the row with no folder.
const markOf = (row: HTMLElement) => {
  const line = row.closest("li") as HTMLElement;
  return line.querySelector("[aria-hidden='true'], button span") as HTMLElement;
};

beforeEach(() => {
  adopt("en");
  showAll();
  ipc.calls = [];
  ipc.state = state();
});

afterEach(() => showAll());

describe("the three stages, as pure answers", () => {
  it("syncs only with its folder, this computer's yes and the switch on", () => {
    expect(agentsLit(state({ consented: true }))).toBe(true);
    expect(agentsLit(state({ folder: null, consented: true }))).toBe(false);
    expect(agentsLit(state())).toBe(false);
    expect(agentsLit(state({ consented: true, allOn: false }))).toBe(false);
  });

  it("syncs at once only a folder this computer already allowed", () => {
    expect(onlySwitchedOff(state({ consented: true, allOn: false }))).toBe(true);
    expect(onlySwitchedOff(state({ allOn: false }))).toBe(false);
    expect(onlySwitchedOff(state({ folder: null, consented: true, allOn: false }))).toBe(false);
    expect(onlySwitchedOff(state({ consented: true }))).toBe(false);
  });

  it("tells the stage from the folder, this computer's answer and what the core counts in it", () => {
    expect(agentsStage("A", state({ consented: true, allOn: false, held: 2 }))).toBe("brought");
    expect(agentsStage("A", state({ consented: true, allOn: true, held: 0 }))).toBe("syncs");
    expect(agentsStage("A", state({ consented: true, allOn: true, held: 5 }))).toBe("syncs");
    expect(agentsStage("A", state({ consented: false, held: 1 }))).toBe("brought");
    expect(agentsStage(null, state({ consented: true, held: 3 }))).toBe("unbrought");
    expect(agentsStage(null, null)).toBe("unbrought");
    expect(agentsStage("A", state({ consented: true, allOn: false, held: 0 }))).toBe("brought");
    expect(agentsStage("A", state({ consented: false, held: 0 }))).toBe("unbrought");
    expect(agentsStage("A", null)).toBeNull();
  });

  it("paints coral only while it syncs", () => {
    expect(agentsPaint("syncs")).toBe("text-hue-coral");
    expect(agentsPaint("brought")).toBe("text-faint");
    expect(agentsPaint("unbrought")).toBe("text-faint");
    expect(agentsPaint(null)).toBe("text-faint");
  });

  it("knows Agents only as a top folder named AGENTS with the robot", () => {
    expect(isAgentsFolder(root)).toBe(true);
    expect(isAgentsFolder({ ...root, parent: "W" })).toBe(false);
    expect(isAgentsFolder({ ...root, icon: null })).toBe(false);
    expect(isAgentsFolder({ ...root, icon: "folder" })).toBe(false);
    expect(isAgentsFolder({ ...root, name: "Agents" })).toBe(false);
    expect(isAgentsFolder({ ...root, name: "agents" })).toBe(false);
  });

  it("shows Agents in the person's language and leaves every other name as it is", () => {
    expect(folderShown(root)).toBe("Agents");
    expect(folderShown(folder("W", "trabajo"))).toBe("trabajo");
    expect(folderShown({ ...root, parent: "W" })).toBe("AGENTS");
    expect(folderShown({ ...root, icon: null })).toBe("AGENTS");
    adopt("es");
    expect(folderShown(root)).toBe("Agentes");
  });
});

describe("the words Agents speaks with", () => {
  it("never says on, off nor the stored name AGENTS", () => {
    // Tisty's own skills and hooks are turned on and off by the owner's word; the folder only syncs.
    const folders = (all: Record<string, string>) =>
      Object.entries(all)
        .filter(
          ([key]) =>
            !key.startsWith("tistySkill") &&
            !key.startsWith("tistyLanding") &&
            !key.startsWith("agentGroupHooksAll"),
        )
        .map(([, said]) => said);
    for (const said of [...folders(agentsEn), ...folders(agentsEs)]) {
      expect(said).not.toMatch(/encend|apag|turn on|turn off/i);
    }
    for (const said of [...Object.values(agentsEn), ...Object.values(agentsEs)]) {
      expect(said).not.toMatch(/AGENTS(?![.\w])/);
    }
  });

  it("names the assistants found here, one, two or more", () => {
    const named = (...names: string[]) =>
      foundSaid(state({ assistants: names.map((name) => ({ id: name, name, on: true })) }));
    expect(named()).toBe("I found no agent on this computer.");
    expect(named("Claude")).toBe("I found Claude on this computer.");
    expect(named("Claude", "Codex")).toBe("I found Claude and Codex on this computer.");
    expect(named("Claude", "Codex", "Gemini")).toBe(
      "I found Claude, Codex and Gemini on this computer.",
    );
    adopt("es");
    expect(named("Claude", "Codex")).toBe("Encontré Claude y Codex en este equipo.");
    expect(named()).toBe("No encontré ningún agente en este equipo.");
  });
});

describe("Agents in the sidebar with nothing brought and no folder", () => {
  it("stands there grey, with no words, and a click opens the first time", async () => {
    const first = listening(AGENTS_FIRST);
    tree(bare);
    const row = screen.getByRole("button", { name: unbroughtName });
    expect(row.textContent).toBe("Agents");
    expect(row.className).toContain("text-faint");
    expect(markOf(row).className).toContain("text-faint");
    expect(markOf(row).className).not.toContain("text-hue-coral");
    expect(document.querySelector("[data-agents]")).toBeNull();
    await userEvent.click(row);
    expect(first.heard).toHaveBeenCalledTimes(1);
    first.off();
  });

  it("opens its menu on a right click, Shift+F10 or the menu key", () => {
    const onAgentsMenu = vi.fn();
    tree(bare, { onAgentsMenu });
    const row = screen.getByRole("button", { name: unbroughtName });
    expect(row.getAttribute("aria-keyshortcuts")).toBe("Shift+F10");
    fireEvent.contextMenu(row, { clientX: 40, clientY: 60 });
    expect(onAgentsMenu).toHaveBeenLastCalledWith({ x: 40, y: 60 });
    fireEvent.keyDown(row, { key: "F10", shiftKey: true });
    fireEvent.keyDown(row, { key: "ContextMenu" });
    expect(onAgentsMenu).toHaveBeenCalledTimes(3);
    fireEvent.keyDown(row, { key: "F10" });
    expect(onAgentsMenu).toHaveBeenCalledTimes(3);
  });

  it("leaves the right click alone when nobody asked for its menu", () => {
    tree(bare);
    const row = screen.getByRole("button", { name: unbroughtName });
    expect(row.getAttribute("aria-keyshortcuts")).toBeNull();
    expect(fireEvent.contextMenu(row)).toBe(true);
    fireEvent.keyDown(row, { key: "ContextMenu" });
  });

  it("is reached with the arrows like any row", () => {
    tree(bare);
    const work = screen.getByRole("button", { name: "trabajo" });
    const agents = screen.getByRole("button", { name: unbroughtName });
    act(() => work.focus());
    fireEvent.keyDown(work, { key: "ArrowDown" });
    expect(document.activeElement).toBe(agents);
    fireEvent.keyDown(agents, { key: "ArrowUp" });
    expect(document.activeElement).toBe(work);
    fireEvent.keyDown(agents, { key: "Enter" });
  });

  it("holds the tab stop when it is the only row before the loose documents", () => {
    tree({ folders: [], docs: [] });
    expect(screen.getByRole("button", { name: unbroughtName }).tabIndex).toBe(0);
  });

  it("goes once hidden, and while the sidebar is sifted", () => {
    const { unmount } = tree(bare, { unfolded: true });
    expect(screen.queryByRole("button", { name: unbroughtName })).toBeNull();
    unmount();
    hide(AGENTS);
    tree(bare);
    expect(screen.queryByRole("button", { name: unbroughtName })).toBeNull();
  });

  it("asks nothing at all without a folder", async () => {
    tree(bare);
    rail(bare);
    await settle();
    expect(sent("agents_state")).toHaveLength(0);
    expect(sent("agents_look")).toHaveLength(0);
  });
});

describe("Agents in the sidebar with an empty folder never allowed here", () => {
  it("is still unbrought: grey, no count, and a click opens the first time", async () => {
    const first = listening(AGENTS_FIRST);
    const onHere = vi.fn();
    tree(empty, { onHere });
    const row = await screen.findByRole("button", { name: unbroughtName });
    expect(row.textContent).toBe("Agents");
    expect(row.className).toContain("text-faint");
    expect(
      screen.getByRole("button", { name: "Open Agents" }).querySelector(".text-faint"),
    ).toBeTruthy();
    expect(document.querySelector(".text-hue-coral")).toBeNull();
    await userEvent.click(row);
    expect(first.heard).toHaveBeenCalledTimes(1);
    expect(onHere).not.toHaveBeenCalled();
    first.off();
  });
});

describe("Agents in the sidebar, brought but not synced", () => {
  it("draws a grey icon and its plain name, says it does not sync, and opens like any folder", async () => {
    ipc.state = state({ consented: true, allOn: false });
    const first = listening(AGENTS_FIRST);
    const onHere = vi.fn();
    tree(held, { onHere });
    const row = await screen.findByRole("button", { name: broughtName });
    expect(row.className).not.toContain("text-faint");
    expect(within(row).getByText("Agents")).toBeTruthy();
    const grip = screen.getByRole("button", { name: "Open Agents" });
    expect(grip.querySelector(".text-faint")).toBeTruthy();
    expect(grip.querySelector(".text-hue-coral")).toBeNull();
    expect(grip.querySelector(".text-hue-purple")).toBeNull();
    await userEvent.click(row);
    expect(onHere).toHaveBeenCalledWith("A");
    expect(first.heard).not.toHaveBeenCalled();
    await userEvent.click(grip);
    expect(screen.getByRole("button", { name: "Doc D1" })).toBeTruthy();
    first.off();
  });

  it("counts as brought when another computer brought it and this one never said yes", async () => {
    ipc.state = state({ consented: false, held: 1 });
    tree(held);
    expect(await screen.findByRole("button", { name: broughtName })).toBeTruthy();
  });
});

describe("Agents in the sidebar while it syncs", () => {
  it("draws the coral icon, its plain name and the count, and no word", async () => {
    ipc.state = state({ consented: true });
    const onHere = vi.fn();
    tree(held, { onHere });
    await settle();
    const row = screen.getByRole("button", { name: "Agents" });
    expect(row.textContent).toBe("Agents3");
    const grip = screen.getByRole("button", { name: "Open Agents" });
    expect(grip.querySelector(".text-hue-coral")).toBeTruthy();
    expect(document.querySelector("[data-agents]")).toBeNull();
    await userEvent.click(row);
    expect(onHere).toHaveBeenCalledWith("A");
    expect(sent("agents_look")).toHaveLength(0);
  });

  it("is drawn grey and plain while its state is still on its way", async () => {
    ipc.state = new Promise(() => {});
    tree(held);
    rail(held);
    await settle();
    expect(screen.getAllByRole("button", { name: "Agents" })).toHaveLength(2);
    expect(
      screen.getByRole("button", { name: "Open Agents" }).querySelector(".text-faint"),
    ).toBeTruthy();
    expect(document.querySelector(".text-hue-coral")).toBeNull();
  });

  it("goes when hidden by its own key or by its folder's", async () => {
    ipc.state = state({ consented: true });
    hide("A");
    const { unmount } = tree(held);
    await settle();
    expect(screen.queryByRole("button", { name: /Agents/ })).toBeNull();
    unmount();
    showAll();
    hide(AGENTS);
    tree(held);
    await settle();
    expect(screen.queryByRole("button", { name: /Agents/ })).toBeNull();
    expect(screen.getByRole("button", { name: "trabajo" })).toBeTruthy();
  });
});

describe("Agents in the folded sidebar", () => {
  const iconOf = (button: HTMLElement) => button.querySelector("span") as HTMLElement;

  it("stands there grey without a folder, and answers with the first time", async () => {
    const first = listening(AGENTS_FIRST);
    const onReveal = vi.fn();
    rail(bare, onReveal);
    const button = screen.getByRole("button", { name: unbroughtName });
    expect(button.getAttribute("title")).toBe("Agents");
    expect(iconOf(button).className).toContain("text-faint");
    await userEvent.click(button);
    expect(first.heard).toHaveBeenCalledTimes(1);
    expect(onReveal).not.toHaveBeenCalled();
    first.off();
  });

  it("opens a brought folder that does not sync, grey and saying so", async () => {
    ipc.state = state({ consented: true, allOn: false });
    const first = listening(AGENTS_FIRST);
    const onReveal = vi.fn();
    rail(held, onReveal);
    const button = await screen.findByRole("button", { name: broughtName });
    expect(iconOf(button).className).toContain("text-faint");
    await userEvent.click(button);
    expect(onReveal).toHaveBeenCalledWith("A");
    expect(first.heard).not.toHaveBeenCalled();
    first.off();
  });

  it("opens a folder that syncs, in coral", async () => {
    ipc.state = state({ consented: true });
    const onReveal = vi.fn();
    rail(held, onReveal);
    await settle();
    const button = screen.getByRole("button", { name: "Agents" });
    expect(iconOf(button).className).toContain("text-hue-coral");
    await userEvent.click(button);
    expect(onReveal).toHaveBeenCalledWith("A");
  });

  it("answers an empty folder never allowed with the first time", async () => {
    const first = listening(AGENTS_FIRST);
    rail(empty);
    await userEvent.click(await screen.findByRole("button", { name: unbroughtName }));
    expect(first.heard).toHaveBeenCalledTimes(1);
    first.off();
  });

  it("leaves Agents out once hidden", async () => {
    hide(AGENTS);
    rail(bare);
    await settle();
    expect(screen.queryByRole("button", { name: /Agents/ })).toBeNull();
  });
});

describe("the Agents folder open in the content panel", () => {
  const opened = (papers: Papers) =>
    render(
      <Folder
        folder={root}
        folders={papers.folders}
        docs={papers.docs}
        onOpen={vi.fn()}
        onHere={vi.fn()}
      />,
    );

  it("says under its title that it does not sync, and its link asks to sync", async () => {
    ipc.state = state({ consented: true, allOn: false });
    const sync = listening(AGENTS_SYNC);
    opened(held);
    const link = await screen.findByRole("button", { name: "Sync" });
    expect(screen.getByText(/Not synced with your agents/)).toBeTruthy();
    expect(screen.getByRole("heading", { level: 1 }).textContent?.trim()).toBe("Agents");
    expect(screen.getByRole("heading", { level: 1 }).querySelector(".text-faint")).toBeTruthy();
    await userEvent.click(link);
    expect(sync.heard).toHaveBeenCalledTimes(1);
    sync.off();
  });

  it("says nothing of syncing while it syncs, and wears its coral", async () => {
    ipc.state = state({ consented: true });
    opened(held);
    await settle();
    expect(screen.queryByText(/Not synced with your agents/)).toBeNull();
    expect(screen.queryByRole("button", { name: "Sync" })).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).querySelector(".text-hue-coral")).toBeTruthy();
  });

  it("asks nothing for a folder that is not Agents, and shows Agents' name below another", async () => {
    const claude = folder("C", "claude", { parent: "A" });
    render(
      <Folder
        folder={claude}
        folders={[root, claude]}
        docs={[]}
        onOpen={vi.fn()}
        onHere={vi.fn()}
      />,
    );
    await settle();
    expect(sent("agents_state")).toHaveLength(0);
    expect(screen.getByRole("button", { name: "Agents" })).toBeTruthy();
    expect(screen.queryByText(/Not synced with your agents/)).toBeNull();
  });
});

describe("a notice with an X", () => {
  it("shows the X only when it can be closed and was asked for", async () => {
    const onClose = vi.fn();
    const { rerender } = render(
      <Modal title="Hola" onClose={onClose}>
        body
      </Modal>,
    );
    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
    rerender(
      <Modal title="Hola" closable>
        body
      </Modal>,
    );
    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
    rerender(
      <Modal title="Hola" closable onClose={onClose}>
        body
      </Modal>,
    );
    const x = screen.getByRole("button", { name: "Close" });
    expect(x.getAttribute("title")).toBe("Close");
    await userEvent.click(x);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("dialog", { name: "Hola" })).toBeTruthy();
  });
});

describe("the menu of an Agents with no folder", () => {
  const stand = () => {
    const given: Record<string, unknown> = {
      tight: false,
      error: null,
      selected: undefined,
      mode: "side",
      chosen: { named: "docs" },
      found: null,
      asideShut: false,
      afoot: false,
      carries: { current: null },
      papers: { folders: [], docs: [] },
      board: null,
      showing: null,
      held: undefined,
      acted: { current: null },
      greet: false,
      leaving: false,
      torn: false,
      asked: false,
      parcels: { asking: false },
      asking: false,
      marking: false,
      setMenu: vi.fn(),
    };
    const spies = new Map<string, ReturnType<typeof vi.fn>>();
    return new Proxy(given, {
      get: (target, key: string) => {
        if (key in target) return target[key];
        if (!spies.has(key)) spies.set(key, vi.fn());
        return spies.get(key);
      },
    });
  };
  const data = { tasks: [] } as unknown as Snapshot;

  it("offers only to hide it, and hiding hides Agents", () => {
    const held = stand();
    const desk = deskOf(held as unknown as Parameters<typeof deskOf>[0], data);
    desk.agentsMenu({ x: 5, y: 7 });
    const menu = (held.setMenu as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(menu.at).toEqual({ x: 5, y: 7 });
    expect(menu.on).toBe(AGENTS);
    expect(menu.label).toBe("Agents");
    expect(menu.choices.map((one: { key: string }) => one.key)).toEqual(["hide"]);
    expect(menu.choices[0].apart).toBe(false);
    menu.choices[0].onPick();
    expect(isHidden(AGENTS)).toBe(true);
  });

  it("asks the stage before Agents' own folder menu opens", async () => {
    ipc.state = state({ consented: true, allOn: false });
    const held = stand();
    (held as Record<string, unknown>).papers = { folders: [root], docs: [filed("D1", "A")] };
    const desk = deskOf(held as unknown as Parameters<typeof deskOf>[0], data);
    desk.folderMenu(root, { x: 1, y: 2 });
    await settle();
    const menu = (held.setMenu as ReturnType<typeof vi.fn>).mock.calls[0][0];
    expect(menu.on).toBe("A");
    expect(menu.choices.map((one: { key: string }) => one.key)).toContain("agentsSync");
  });
});
