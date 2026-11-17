import { describe, expect, it, vi } from "vitest";
import { type Acting, actions, sifted } from "../commanding";
import type { Hands } from "../docMenus";
import { WHOLE_ASKED } from "../ui/wholeKeys";

const hands = (showing: string | null = null): Hands =>
  ({
    papers: { folders: [], docs: [] },
    showing,
    newDoc: vi.fn(),
    newDiagram: vi.fn(),
    bringIn: vi.fn(),
    makeFolder: vi.fn(),
    parcels: { takeParcel: vi.fn(), packUp: vi.fn() },
  }) as unknown as Hands;

const acting = (more: Partial<Acting> = {}): Acting => ({
  chosen: { named: "tasks" },
  here: null,
  hands: hands(),
  board: "board-file",
  folded: false,
  choose: vi.fn(),
  library: vi.fn(),
  diary: vi.fn(),
  capture: vi.fn(),
  find: vi.fn(),
  fold: vi.fn(),
  ...more,
});

const named = (all: { label: string }[], label: string) => {
  const one = all.find((each) => each.label === label);
  if (!one) throw new Error(`no action called ${label}`);
  return one as { label: string; onPick: () => void; keys?: string };
};

describe("the window's own actions", () => {
  it("are about twenty, each with its own key", () => {
    const all = actions(acting());

    expect(all.length).toBeGreaterThanOrEqual(18);
    expect(all.length).toBeLessThanOrEqual(24);
    expect(new Set(all.map((one) => one.key)).size).toBe(all.length);
  });

  it("say their shortcut where they have one", () => {
    const all = actions(acting());

    expect(named(all, "Help").keys).toBe("F1");
    expect(named(all, "Fold the sidebar").keys).toBe("Ctrl+B");
    expect(named(all, "Search tasks, documents, lists…").keys).toBe("Ctrl+K");
  });

  it("go where the sidebar goes", () => {
    const a = acting();
    const all = actions(a);

    named(all, "Priorities").onPick();
    named(all, "Upcoming").onPick();
    named(all, "Brainstorm").onPick();
    named(all, "Weekly review").onPick();

    expect(a.choose).toHaveBeenCalledWith({ named: "quadrants" });
    expect(a.choose).toHaveBeenCalledWith({ named: "tasks", slice: "upcoming" });
    expect(a.choose).toHaveBeenCalledWith({ named: "docs", doc: "board-file" });
    expect(a.choose).toHaveBeenCalledWith({ named: "weekly" });
  });

  it("make things with the same hands as the library's menu", () => {
    const a = acting({ here: "F1" });

    named(actions(a), "New document").onPick();

    expect(a.hands.newDoc).toHaveBeenCalledWith("F1");
  });

  it("offer to unfold a folded sidebar", () => {
    expect(() => named(actions(acting({ folded: true })), "Unfold the sidebar")).not.toThrow();
  });

  it("leave the Brainstorm out until there is one", () => {
    const all = actions(acting({ board: null }));

    expect(all.some((one) => one.key === "brainstorm")).toBe(false);
  });

  it("offer the document's own only while one is open", () => {
    expect(actions(acting()).some((one) => one.key === "whole")).toBe(false);

    const a = acting({ chosen: { named: "docs", doc: "file-a" }, hands: hands("file-a") });
    const heard = vi.fn();
    window.addEventListener(WHOLE_ASKED, heard);
    named(actions(a), "Switch between Rich text and Markdown").onPick();
    window.removeEventListener(WHOLE_ASKED, heard);

    expect(heard).toHaveBeenCalledOnce();
  });
});

describe("sifting the actions by what is typed after >", () => {
  it("keeps all of them when nothing is typed", () => {
    const all = actions(acting());

    expect(sifted(all, " ")).toBe(all);
  });

  it("reads the group too, so «go» finds the places", () => {
    const found = sifted(actions(acting()), "go arch");

    expect(found.map((one) => one.label)).toEqual(["Archive"]);
  });
});
