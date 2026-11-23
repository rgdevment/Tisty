import { act, render, renderHook, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { dropAsked } from "../agentsDropping";
import type { Reach } from "../agentsSync";
import type { Filed, Folded } from "../core";
import { fill, t } from "../locales";
import { asksSure } from "../sure";
import { dropAllAsked, goingWith } from "../tidying";
import { tonguesNamed } from "../tongues";
import AgentsDropping from "../ui/AgentsDropping";
import { namesJoined, waitingOf } from "../ui/AgentsReview";
import { usePapers } from "../usePapers";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  reach: null as unknown,
  dropped: null as unknown,
  reachFails: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    switch (cmd) {
      case "docs":
        return Promise.resolve({ folders: FOLDERS, docs: DOCS });
      case "agents_reach":
        return ipc.reachFails
          ? Promise.reject({ code: "cannotWrite" })
          : Promise.resolve(ipc.reach);
      case "agents_drop":
        return Promise.resolve(ipc.dropped);
      default:
        return Promise.resolve(null);
    }
  },
}));

const folder = (id: string, name: string, parent: string | null, icon: string | null): Folded => ({
  id,
  name,
  parent,
  icon,
  holds: 0,
  archived: false,
  away: false,
});

const FOLDERS = [
  folder("A", "AGENTS", null, "robot"),
  folder("G", "Global", "A", null),
  folder("N", "Notes", null, null),
];

const doc = (id: string, title: string, at: string | null, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: at,
  archived: false,
  away: false,
  ...more,
});

const SKILL = doc("S1", "voice", "G", { lang: "en", tongue: ["S1", 0] });
const SKILL_ES = doc("S2", "voz", "G", { lang: "es", tongue: ["S1", 1] });
const RULE = doc("R1", "tone", "G");
const PAGE = doc("P1", "examples", null, { pageOf: "R1" });
const NOTE = doc("N1", "Groceries", "N");
const DOCS = [SKILL, SKILL_ES, RULE, PAGE, NOTE];

const SHARED = "/h/.agents/skills/voice/SKILL.md";
const CLAUDE = "/h/.claude/skills/voice/SKILL.md";
const EVERYWHERE: Reach = {
  at: [CLAUDE, SHARED],
  agents: ["Claude", "Codex", "Copilot"],
  changed: [],
  paused: [],
  rewritten: [],
};

const called = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const hands = () => ({
  chosen: { named: "docs" as const },
  setChosen: vi.fn(),
  here: undefined,
  setHere: vi.fn(),
  setReturning: vi.fn(),
  setBacking: vi.fn(),
  setError: vi.fn(),
  openTask: vi.fn(),
  noted: vi.fn(),
  lookForAStar: vi.fn(),
  carries: { current: null },
  paging: { current: null },
});

const loaded = async (given = hands()) => {
  const hook = renderHook(() => usePapers(given));
  await waitFor(() => expect(hook.result.current.papers.docs).toHaveLength(DOCS.length));
  return { hook, given };
};

const said = (one: Filed) => {
  const [going] = goingWith([one], DOCS);
  if (going.tongues.length > 1) {
    return fill("dropTonguesSure", one.title, tonguesNamed(going.tongues));
  }
  return fill(going.pages.length ? "dropPagesSure" : "dropDocSure", one.title);
};

let plain: string[] = [];

beforeEach(() => {
  ipc.calls = [];
  ipc.reach = EVERYWHERE;
  ipc.reachFails = false;
  ipc.dropped = { gone: ["S1"], refused: [], untaken: [], unmarked: [] };
  plain = [];
  asksSure((asked) => {
    plain.push(asked);
    return Promise.resolve(true);
  });
});

afterEach(() => asksSure(null));

const dropping = async (one: Filed, answer: string) => {
  render(<AgentsDropping />);
  const { hook, given } = await loaded();
  let going: Promise<void> | undefined;
  act(() => {
    going = hook.result.current.dropDoc(one);
  });
  await userEvent.click(await screen.findByRole("button", { name: answer }));
  await act(() => going);
  return given;
};

describe("deleting a skill or rule of Agents", () => {
  it("asks once, saying what goes with it as ever and naming its agents", async () => {
    render(<AgentsDropping />);
    const { hook } = await loaded();
    let going: Promise<void> | undefined;
    act(() => {
      going = hook.result.current.dropDoc(SKILL);
    });

    expect(await screen.findByText(said(SKILL))).toBeTruthy();
    expect(screen.getByText(fill("agentsDropAsk", "Claude, Codex and Copilot"))).toBeTruthy();
    expect(screen.getByText(t("agentsDropBackup"))).toBeTruthy();
    expect(called("agents_reach")[0].args).toEqual({ ids: ["S1"] });
    await userEvent.click(screen.getByRole("button", { name: t("agentsDropAway") }));
    await act(() => going);

    expect(called("agents_drop").map((one) => one.args)).toEqual([
      { ids: ["S1"], away: true, at: [CLAUDE, SHARED] },
    ]);
    expect(called("doc_drop")).toEqual([]);
    expect(plain, "the one question takes the place of being sure").toEqual([]);
  });

  it("names the languages and the pages that go with it, as the plain question did", async () => {
    expect(said(SKILL)).toBe(fill("dropTonguesSure", "voice", tonguesNamed([SKILL, SKILL_ES])));
    render(<AgentsDropping />);
    const { hook } = await loaded();
    act(() => {
      void hook.result.current.dropDoc(RULE);
    });
    expect(await screen.findByText(fill("dropPagesSure", "tone"))).toBeTruthy();
  });

  it("«Only from Tisty» deletes it and hands back the very paths it showed", async () => {
    await dropping(SKILL, t("agentsDropOnlyTisty"));
    expect(called("agents_drop").map((one) => one.args)).toEqual([
      { ids: ["S1"], away: false, at: [CLAUDE, SHARED] },
    ]);
  });

  it("cancelling deletes nothing and takes nothing away", async () => {
    await dropping(SKILL, t("cancel"));
    expect(called("agents_drop")).toEqual([]);
    expect(called("doc_drop")).toEqual([]);
  });

  it("says apart a changed copy, paused agents and a file written again", async () => {
    ipc.reach = {
      at: [CLAUDE],
      agents: ["Claude"],
      changed: ["Codex"],
      paused: ["Copilot", "Gemini"],
      rewritten: ["Codex"],
    };
    render(<AgentsDropping />);
    const { hook } = await loaded();
    act(() => {
      void hook.result.current.dropDoc(SKILL);
    });
    expect(await screen.findByText(fill("agentsDropAsk", "Claude"))).toBeTruthy();
    expect(screen.getByText(fill("agentsDropChanged", "Codex"))).toBeTruthy();
    expect(screen.getByText(fill("agentsDropPausedMany", "Copilot and Gemini"))).toBeTruthy();
    expect(screen.getByText(fill("agentsDropRewritten", "Codex"))).toBeTruthy();
  });

  it("still asks when every copy is changed or paused, offering only to delete it from Tisty", async () => {
    ipc.reach = { at: [], agents: [], changed: [], paused: ["Codex"], rewritten: [] };
    render(<AgentsDropping />);
    const { hook } = await loaded();
    act(() => {
      void hook.result.current.dropDoc(SKILL);
    });
    expect(await screen.findByText(fill("agentsDropPaused", "Codex"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("agentsDropAway") })).toBeNull();
    expect(screen.queryByText(t("agentsDropBackup"))).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("agentsDropOnlyTisty") }));
    await waitFor(() =>
      expect(called("agents_drop").map((one) => one.args)).toEqual([
        { ids: ["S1"], away: false, at: [] },
      ]),
    );
  });

  it("several at once are one question that lists them as the plain one did", async () => {
    ipc.dropped = { gone: ["S1", "R1"], refused: [], untaken: [], unmarked: [] };
    render(<AgentsDropping />);
    const { hook } = await loaded();
    let going: Promise<void> | undefined;
    act(() => {
      going = hook.result.current.dropDocs([SKILL, RULE]);
    });
    const listed = dropAllAsked(goingWith([SKILL, RULE], DOCS));
    expect(
      await screen.findByText((_, node) => node?.tagName === "P" && node.textContent === listed),
    ).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("agentsDropAway") }));
    await act(() => going);
    expect(called("agents_reach")[0].args).toEqual({ ids: ["S1", "R1"] });
    expect(called("agents_drop").map((one) => one.args)).toEqual([
      { ids: ["S1", "R1"], away: true, at: [CLAUDE, SHARED] },
    ]);
    expect(plain).toEqual([]);
  });

  it("says what was not deleted, taken away or noted, never in silence", async () => {
    ipc.dropped = {
      gone: [],
      refused: [{ id: "S1", refusal: { code: "documentLocked" } }],
      untaken: [{ at: CLAUDE, refusal: { code: "agentLinked" } }],
      unmarked: [{ at: SHARED, refusal: { code: "cannotWrite" } }],
    };
    const given = await dropping(SKILL, t("agentsDropAway"));
    const told = given.setError.mock.calls[given.setError.mock.calls.length - 1]?.[0] as string;
    expect(told).toContain("voice");
    expect(told).toContain(CLAUDE);
    expect(told).toContain(SHARED);
  });

  it("deletes nothing when Agents cannot say who keeps a copy", async () => {
    ipc.reachFails = true;
    const { hook, given } = await loaded();
    await act(() => hook.result.current.dropDoc(SKILL));
    expect(given.setError).toHaveBeenCalled();
    expect(called("doc_drop")).toEqual([]);
    expect(called("agents_drop")).toEqual([]);
    expect(plain).toEqual([]);
  });

  it("with no window to ask in, the plain question deletes it from Tisty alone", async () => {
    const { hook } = await loaded();
    await act(() => hook.result.current.dropDoc(SKILL));
    expect(plain).toEqual([said(SKILL)]);
    expect(called("agents_drop").map((one) => one.args)).toEqual([
      { ids: ["S1"], away: false, at: [CLAUDE, SHARED] },
    ]);
  });

  it("a second question waits for the first instead of taking its place", async () => {
    render(<AgentsDropping />);
    let first: Promise<unknown> | undefined;
    let second: Promise<unknown> | undefined;
    act(() => {
      first = dropAsked({ said: "First?", reach: EVERYWHERE });
      second = dropAsked({ said: "Second?", reach: EVERYWHERE });
    });
    expect(await screen.findByText("First?")).toBeTruthy();
    expect(screen.queryByText("Second?")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: t("cancel") }));
    await expect(first).resolves.toBeNull();
    expect(await screen.findByText("Second?")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("agentsDropOnlyTisty") }));
    await expect(second).resolves.toBe("tisty");
  });
});

describe("deleting what no agent keeps", () => {
  it("a document outside Agents asks as ever and never asks Agents", async () => {
    const { hook } = await loaded();
    await act(() => hook.result.current.dropDoc(NOTE));
    expect(called("agents_reach")).toEqual([]);
    expect(plain).toEqual([fill("dropDocSure", "Groceries")]);
    expect(called("doc_drop").map((one) => one.args)).toEqual([{ id: "N1" }]);
  });

  it("a document of Agents whose files others still make up asks as ever", async () => {
    ipc.reach = { at: [], agents: [], changed: [], paused: [], rewritten: ["Claude"] };
    const { hook } = await loaded();
    await act(() => hook.result.current.dropDoc(SKILL));
    expect(plain).toEqual([said(SKILL)]);
    expect(called("doc_drop").map((one) => one.args)).toEqual([{ id: "S1" }]);
    expect(called("agents_drop")).toEqual([]);
  });
});

describe("files the person let stay", () => {
  const loose = (stayed: boolean) => ({
    at: "/h/a.md",
    assistant: "claude",
    changed: false,
    stayed,
  });

  it("are asked about only when the person opens them", () => {
    const told = { both: [], loose: [loose(true), loose(false)], unled: [] };
    expect(waitingOf(told)).toHaveLength(1);
    expect(waitingOf(told, true)).toHaveLength(2);
  });

  it("names one, two or three agents as a sentence says them", () => {
    expect(namesJoined([])).toBe("");
    expect(namesJoined(["Claude"])).toBe("Claude");
    expect(namesJoined(["Claude", "Codex"])).toBe("Claude and Codex");
    expect(namesJoined(["Claude", "Codex", "Gemini"])).toBe("Claude, Codex and Gemini");
  });
});
