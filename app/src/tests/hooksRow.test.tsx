import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Wired } from "../core";
import { adopt } from "../locales";
import TistyHooksRow, { hooksStanding, tistyHookLines } from "../ui/HooksRow";

const agent = (id: string, name: string, more: Partial<Wired> = {}): Wired => ({
  id,
  name,
  at: `/home/${id}`,
  wired: true,
  astray: false,
  maybe: false,
  byHand: false,
  manual: { file: `${id}.json`, snippet: "{}" },
  hooks: { on: true, at: `/home/${id}/hooks.json` },
  ...more,
});

const off = { on: false, at: "/home/x/hooks.json" };

const row = (agents: Wired[] | null, more: Partial<Parameters<typeof TistyHooksRow>[0]> = {}) =>
  render(
    <ul>
      <TistyHooksRow agents={agents} depth={1} step={12} onOpen={vi.fn()} {...more} />
    </ul>,
  );

beforeEach(() => adopt("en"));

describe("Tisty's hooks as one row", () => {
  it("is one button with the name and, on the right, where they run", () => {
    row([agent("claude", "Claude"), agent("codex", "Codex")]);
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(1);
    expect(buttons[0].textContent).toBe("Tisty's hooksOn");
  });

  it("names every agent and its state in the label and the title", () => {
    row([agent("claude", "Claude"), agent("codex", "Codex", { hooks: off })]);
    const said = "Tisty's hooks: Claude: with hooks; Codex: no hooks";
    const button = screen.getByRole("button", { name: said });
    expect(button.getAttribute("title")).toBe(`${said} · Open in Settings › Agents › Connected`);
  });

  it("draws nothing when no connected agent has hooks, or they are not read yet", () => {
    for (const none of [
      [],
      null,
      [agent("gemini", "Gemini", { wired: false })],
      [agent("cursor", "Cursor", { hooks: null })],
    ]) {
      const { container, unmount } = row(none);
      expect(container.querySelectorAll("li")).toHaveLength(0);
      unmount();
    }
  });

  it("hides an agent that is not connected or has no hooks to read", () => {
    row([
      agent("claude", "Claude"),
      agent("gemini", "Gemini", { wired: false }),
      agent("cursor", "Cursor", { hooks: null }),
      agent("copilot", "Copilot", { hooks: undefined }),
    ]);
    expect(screen.getByRole("button").getAttribute("aria-label")).toBe(
      "Tisty's hooks: Claude: with hooks",
    );
  });

  it("opens Settings when pressed", () => {
    const onOpen = vi.fn();
    row([agent("claude", "Claude")], { onOpen });
    fireEvent.click(screen.getByRole("button"));
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("adds the Codex approval to the title when Codex has its hooks on", () => {
    row([agent("claude", "Claude"), agent("codex", "Codex")]);
    const title = screen.getByRole("button").getAttribute("title") ?? "";
    expect(title).toContain("Approve it in Codex with /hooks");
    expect(title.endsWith("Open in Settings › Agents › Connected")).toBe(true);
  });

  it("says it in Spanish too, for an id that ends in codex", () => {
    adopt("es");
    row([agent("openai-codex", "Codex")]);
    expect(screen.getByRole("button").getAttribute("title")).toContain(
      "Apruébalo en Codex con /hooks",
    );
  });

  it("leaves the approval out when Codex is off or not there", () => {
    const off_ = row([agent("claude", "Claude"), agent("codex", "Codex", { hooks: off })]);
    expect(screen.getByRole("button").getAttribute("title")).not.toContain("/hooks");
    off_.unmount();
    row([agent("claude", "Claude")]);
    expect(screen.getByRole("button").getAttribute("title")).not.toContain("/hooks");
  });

  it("carries no line about approving hooks by hand in Codex", () => {
    row([agent("claude", "Claude"), agent("codex", "Codex")]);
    expect(screen.queryByText(/\/hooks/)).toBeNull();
  });

  it("speaks Spanish when the app does", () => {
    adopt("es");
    row([agent("claude", "Claude", { hooks: off }), agent("codex", "Codex", { hooks: off })]);
    const button = screen.getByRole("button", {
      name: "Hooks de Tisty: Claude: sin hooks; Codex: sin hooks",
    });
    expect(button.textContent).toBe("Hooks de TistyApagados");
  });
});

describe("what the right side of the row says", () => {
  const line = (id: string, name: string, on: boolean) => ({ id, name, on });

  it("says on when all run them, off when none, and which ones when mixed", () => {
    expect(hooksStanding([line("a", "Claude", true), line("b", "Codex", true)])).toEqual({
      said: "On",
      on: true,
    });
    expect(hooksStanding([line("a", "Claude", false), line("b", "Codex", false)])).toEqual({
      said: "Off",
      on: false,
    });
    expect(hooksStanding([line("a", "Claude", true), line("b", "Codex", false)])).toEqual({
      said: "Only in Claude",
      on: true,
    });
    expect(
      hooksStanding([
        line("a", "Claude", true),
        line("b", "Codex", true),
        line("c", "Gemini", false),
      ]).said,
    ).toBe("Only in Claude, Codex");
  });

  it("counts a plugged agent as on and prints the standing in green when some run them", () => {
    row([
      agent("claude", "Claude", { hooks: { on: false, plugged: true, at: "x" } }),
      agent("codex", "Codex", { hooks: off }),
    ]);
    expect(screen.getByText("Only in Claude").className).toContain("text-hue-green");
  });

  it("prints Off in faint, and the mixed standing in Spanish", () => {
    const first = row([agent("claude", "Claude", { hooks: off })]);
    expect(screen.getByText("Off").className).toContain("text-faint");
    first.unmount();
    adopt("es");
    row([agent("claude", "Claude"), agent("codex", "Codex", { hooks: off })]);
    expect(screen.getByText("Solo en Claude")).toBeTruthy();
  });
});

describe("tistyHookLines", () => {
  it("gives one line per connected agent with hooks, on when switched on or plugged", () => {
    expect(
      tistyHookLines([
        agent("claude", "Claude"),
        agent("codex", "Codex", { hooks: off }),
        agent("gemini", "Gemini", { hooks: { on: false, plugged: true, at: "x" } }),
        agent("cursor", "Cursor", { hooks: null }),
        agent("copilot", "Copilot", { wired: false }),
      ]),
    ).toEqual([
      { id: "claude", name: "Claude", on: true },
      { id: "codex", name: "Codex", on: false },
      { id: "gemini", name: "Gemini", on: true },
    ]);
  });

  it("gives none when the agents are not read yet", () => {
    expect(tistyHookLines(null)).toEqual([]);
    expect(tistyHookLines([])).toEqual([]);
  });
});

describe("the hooks row as a row of the tree", () => {
  it("passes what rowOf gives, keyed tisty-hooks", () => {
    const rowOf = vi.fn((key: string) => ({ "data-row": key, tabIndex: -1 }));
    row([agent("claude", "Claude"), agent("codex", "Codex")], { rowOf });
    const button = screen.getByRole("button");
    expect(button.getAttribute("data-row")).toBe("tisty-hooks");
    expect(button.tabIndex).toBe(-1);
    expect(rowOf).toHaveBeenCalledWith("tisty-hooks");
    expect(rowOf).toHaveBeenCalledTimes(1);
  });

  it("is no row without rowOf", () => {
    row([agent("claude", "Claude")]);
    expect(screen.getByRole("button").hasAttribute("data-row")).toBe(false);
  });
});
