import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AGENTS_CHANGED, AGENTS_RESCAN } from "../agentsSync";
import "../agentsWords";
import { adopt } from "../locales";

const ipc = vi.hoisted(() => ({
  takeable: [] as string[],
  calls: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd !== "agents_takeable") return Promise.resolve(null);
    ipc.calls += 1;
    return Promise.resolve([...ipc.takeable]);
  },
}));

const SEEN_KEY = "tisty:agents-seen";
const mount = async () => {
  vi.resetModules();
  const { default: AgentsRefresh, LOOK_EVERY_MS } = await import("../ui/AgentsRefresh");
  const view = render(<AgentsRefresh />);
  await act(async () => {});
  return { view, LOOK_EVERY_MS };
};

const focusAfter = async (ms: number) => {
  vi.setSystemTime(Date.now() + ms);
  await act(async () => {
    window.dispatchEvent(new Event("focus"));
  });
};

describe("the ⟳ button in the Agents folder", () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-11-26T10:00:00Z"));
    localStorage.clear();
    ipc.takeable = ["/h/.claude/skills/pdf/SKILL.md"];
    ipc.calls = 0;
    adopt("en");
  });
  afterEach(() => vi.useRealTimers());

  it("seeds the baseline on first mount and shows no count", async () => {
    await mount();
    expect(ipc.calls).toBe(1);
    expect(screen.queryByText(/new$/)).toBeNull();
    expect(JSON.parse(localStorage.getItem(SEEN_KEY) ?? "{}")).toEqual({
      names: ["/h/.claude/skills/pdf/SKILL.md"],
    });
  });

  it("counts a new path on a focus past the minute", async () => {
    const { LOOK_EVERY_MS } = await mount();
    ipc.takeable = ["/h/.claude/skills/pdf/SKILL.md", "/h/.claude/skills/xlsx/SKILL.md"];
    await focusAfter(LOOK_EVERY_MS);
    expect(ipc.calls).toBe(2);
    expect(screen.getByText("1 new")).toBeTruthy();
  });

  it("does not ask again on a focus within the minute", async () => {
    const { LOOK_EVERY_MS } = await mount();
    await focusAfter(LOOK_EVERY_MS - 1);
    expect(ipc.calls).toBe(1);
  });

  it("says how many in plural", async () => {
    const { LOOK_EVERY_MS } = await mount();
    ipc.takeable = [
      ...ipc.takeable,
      "/h/.claude/skills/a/SKILL.md",
      "/h/.claude/skills/b/SKILL.md",
    ];
    await focusAfter(LOOK_EVERY_MS);
    expect(screen.getByText("2 new")).toBeTruthy();
  });

  it("clicking keeps the baseline, clears the count and asks to look again", async () => {
    const { LOOK_EVERY_MS } = await mount();
    ipc.takeable = ["/h/.claude/skills/pdf/SKILL.md", "/h/.claude/skills/xlsx/SKILL.md"];
    await focusAfter(LOOK_EVERY_MS);
    const rescan = vi.fn();
    window.addEventListener(AGENTS_RESCAN, rescan);
    fireEvent.click(screen.getByRole("button"));
    window.removeEventListener(AGENTS_RESCAN, rescan);
    expect(rescan).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("1 new")).toBeNull();
    expect(JSON.parse(localStorage.getItem(SEEN_KEY) ?? "{}").names).toEqual(ipc.takeable);
  });

  it("recomputes from the last list, without asking again, when the agents change", async () => {
    const { LOOK_EVERY_MS } = await mount();
    ipc.takeable = ["/h/.claude/skills/pdf/SKILL.md", "/h/.claude/skills/xlsx/SKILL.md"];
    await focusAfter(LOOK_EVERY_MS);
    await act(async () => {
      window.dispatchEvent(new Event(AGENTS_CHANGED));
    });
    expect(ipc.calls).toBe(2);
    expect(screen.getByText("1 new")).toBeTruthy();
  });
});
