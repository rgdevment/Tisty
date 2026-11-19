import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsLook, AgentsState } from "../agentsSync";
import type { Folded, Papers } from "../core";
import { AGENTS, BRAINSTORM, DIARY, hide, isHidden, showAll } from "../hiding";
import { adopt } from "../locales";
import HiddenFolders, { hiddenRows } from "../ui/HiddenFolders";

const seen = vi.hoisted(() => ({
  look: null as unknown,
  state: null as unknown,
  asked: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    seen.asked.push(cmd);
    if (cmd === "agents_state") return Promise.resolve(seen.state);
    return Promise.resolve(cmd === "agents_look" ? seen.look : []);
  },
}));

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

const papers: Papers = {
  folders: [
    folder("A", "AGENTS", { icon: "robot" }),
    folder("W", "trabajo"),
    folder("C", "claude", { parent: "W" }),
    folder("O", "viejo", { away: true }),
  ],
  docs: [],
};

const look = (more: Partial<AgentsLook>): AgentsLook => ({
  folder: "A",
  docs: 1,
  changes: 0,
  waiting: 0,
  unagreed: 0,
  proposed: 0,
  consented: true,
  allOn: true,
  asks: false,
  assistants: [],
  both: [],
  loose: [],
  unled: [],
  ...more,
});

const lit: AgentsState = { folder: "A", consented: true, allOn: true, assistants: [] };

const settle = () => act(() => new Promise((ready) => setTimeout(ready, 0)));
const listed = () => within(screen.getByRole("list", { name: "Hidden folders" }));

beforeEach(() => {
  adopt("en");
  showAll();
  seen.asked = [];
  seen.state = lit;
  seen.look = look({});
});

describe("the hidden folders in Settings", () => {
  it("says so in one line when nothing is hidden", () => {
    render(<HiddenFolders papers={papers} />);
    expect(listed().getByText("No folder is hidden.")).toBeTruthy();
  });

  it("names each one by its whole path, the diary and Ideas too", () => {
    hide("C");
    hide(DIARY);
    hide(BRAINSTORM);
    render(<HiddenFolders papers={papers} />);
    expect(listed().getByText("trabajo / claude")).toBeTruthy();
    expect(listed().getByText("Diary")).toBeTruthy();
    expect(listed().getByText("Ideas")).toBeTruthy();
  });

  it("brings one back to the sidebar and leaves the others hidden", async () => {
    hide("W");
    hide(DIARY);
    render(<HiddenFolders papers={papers} />);
    await userEvent.click(
      listed().getByRole("button", { name: "Show trabajo in the sidebar again" }),
    );
    expect(isHidden("W")).toBe(false);
    expect(isHidden(DIARY)).toBe(true);
    expect(listed().queryByText("trabajo")).toBeNull();
  });

  it("names the ideas sheet by its own title when it has one", () => {
    const withIdeas: Papers = {
      ...papers,
      docs: [
        { id: "I", file: "ideas-1", title: "Ideas", folder: null, archived: false, away: false },
      ],
    };
    expect(hiddenRows([BRAINSTORM], withIdeas, "ideas-1")).toEqual([
      { keys: [BRAINSTORM], name: "Ideas" },
    ]);
  });

  it("skips a folder that is gone or put away", () => {
    hide("gone");
    hide("O");
    expect(hiddenRows(["gone", "O"], papers)).toEqual([]);
  });

  it("lists AGENTS once, whether hidden by its own key, its folder's or both", () => {
    expect(hiddenRows([AGENTS], papers)).toEqual([
      { keys: [AGENTS, "A"], name: "AGENTS", agents: true },
    ]);
    expect(hiddenRows(["A", AGENTS], papers)).toHaveLength(1);
    expect(hiddenRows([AGENTS], { folders: [], docs: [] })).toEqual([
      { keys: [AGENTS], name: "AGENTS", agents: true },
    ]);
  });

  it("shows AGENTS again under every key it was hidden by", async () => {
    hide("A");
    hide(AGENTS);
    render(<HiddenFolders papers={papers} />);
    await userEvent.click(
      listed().getByRole("button", { name: "Show AGENTS in the sidebar again" }),
    );
    expect(isHidden("A")).toBe(false);
    expect(isHidden(AGENTS)).toBe(false);
  });

  it("names a hidden AGENTS and nothing more, since hidden it is off", async () => {
    hide(AGENTS);
    seen.look = look({ changes: 1 });
    render(<HiddenFolders papers={papers} />);
    await settle();
    expect(listed().getByText("AGENTS")).toBeTruthy();
    expect(listed().queryByText("1 changed")).toBeNull();
    expect(seen.asked.filter((cmd) => cmd.startsWith("agents_"))).toEqual([]);
  });
});
