import { beforeEach, describe, expect, it, vi } from "vitest";

const core = vi.hoisted(() => ({
  answer: null as null | ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) =>
    core.answer ? core.answer(cmd, args) : Promise.resolve(null),
}));

import type { Filed, Folded, Papers } from "../core";
import { docPlace, folderPlace, placeOf, placesRules, type Ruling, rulesOf } from "../places";

const folder = (
  id: string,
  name: string,
  parent: string | null = null,
  icon: string | null = null,
) => ({ id, name, parent, icon, holds: 0, archived: false, away: false }) as Folded;

const paper = (more: Partial<Filed> & { id: string; file: string }): Filed =>
  ({ title: "t", folder: null, archived: false, away: false, ...more }) as Filed;

const root = folder("a1", "AGENTS", null, "robot");
const claude = folder("a2", "Claude", "a1");
const casa = folder("f1", "Casa");

const board = paper({ id: "i1", file: "ideas-1" });
const book = paper({ id: "b1", file: "diary-2026", diary: true });
const day = paper({ id: "p1", file: "diary-day", pageOf: "b1" });
const rule = paper({ id: "r1", file: "rule-1", folder: "a2" });
const plain = paper({ id: "d1", file: "doc-1", folder: "f1" });

const papers = { folders: [casa, root, claude], docs: [board, book, day, rule, plain] } as Papers;

const OPEN_RULES = {
  makes: ["doc", "diagram", "folder", "import", "parcel"],
  pages: true,
  moves: true,
  copies: true,
  goes: true,
};

describe("which place a document or a folder is in", () => {
  it("is Ideas for the pinned sheet", () => {
    expect(placeOf(board, papers, board.file)).toBe("ideas");
  });

  it("is a plain place for the same sheet when nothing is pinned", () => {
    expect(placeOf(board, papers)).toBe("plain");
    expect(placeOf(board, papers, "other-file")).toBe("plain");
  });

  it("is the diary for a book and for a page of it", () => {
    expect(placeOf(book, papers)).toBe("diary");
    expect(placeOf(day, papers)).toBe("diary");
  });

  it("is Agents for a document in an agent's folder, and a plain place elsewhere", () => {
    expect(placeOf(rule, papers)).toBe("agents");
    expect(placeOf(plain, papers)).toBe("plain");
  });

  it("is Agents for the folders inside it, the root included", () => {
    expect(folderPlace("a1", papers)).toBe("agents");
    expect(folderPlace("a2", papers)).toBe("agents");
    expect(folderPlace("f1", papers)).toBe("plain");
    expect(folderPlace(undefined, papers)).toBe("plain");
    expect(folderPlace(null, papers)).toBe("plain");
  });
});

describe("the rules of each place", () => {
  beforeEach(() => {
    core.answer = null;
  });

  it("offers everything in every place until the core has answered", () => {
    for (const place of ["plain", "ideas", "diary", "agents"] as const) {
      expect(rulesOf(place)).toEqual(OPEN_RULES);
    }
  });

  it("asks the core once and then follows what it said", async () => {
    const said: Ruling = {
      plain: OPEN_RULES,
      agents: { ...OPEN_RULES, makes: ["doc"], moves: false },
      ideas: { makes: [], pages: false, moves: false, copies: false, goes: false },
      diary: { makes: [], pages: false, moves: false, copies: true, goes: true },
    } as Ruling;
    const asked: string[] = [];
    core.answer = (cmd) => {
      asked.push(cmd);
      return Promise.resolve(said);
    };

    await placesRules();
    await placesRules();

    expect(asked).toEqual(["places_rules"]);
    expect(rulesOf("ideas").pages).toBe(false);
    expect(rulesOf("ideas").goes).toBe(false);
    expect(rulesOf("diary").copies).toBe(true);
    expect(rulesOf("agents").makes).toEqual(["doc"]);
    expect(rulesOf("plain")).toEqual(OPEN_RULES);
  });
});

describe("the place of a file the core knows", () => {
  it("asks the core for it by file", async () => {
    const calls: { cmd: string; args?: Record<string, unknown> }[] = [];
    core.answer = (cmd, args) => {
      calls.push({ cmd, args });
      return Promise.resolve({ place: "agents", cards: false });
    };

    expect(await docPlace("rule-1")).toEqual({ place: "agents", cards: false });
    expect(calls).toEqual([{ cmd: "doc_place", args: { file: "rule-1" } }]);
  });
});
