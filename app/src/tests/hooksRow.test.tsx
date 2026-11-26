import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Wired } from "../core";
import { adopt } from "../locales";
import HooksRow, { tistyHookLines } from "../ui/HooksRow";

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

beforeEach(() => adopt("en"));

describe("the hooks row of Global", () => {
  it("says for each connected agent whether Tisty's hooks are on or off here", () => {
    render(
      <HooksRow
        agents={[
          agent("claude", "Claude"),
          agent("codex", "Codex", { hooks: { on: false, at: "/home/codex/hooks.json" } }),
        ]}
        onOpen={vi.fn()}
      />,
    );
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(2);
    expect(buttons[0].textContent).toBe("Claude on this computer");
    expect(buttons[1].textContent).toBe("Codex on this computer");
  });

  it("hides an agent that is not connected or has no hooks to read", () => {
    render(
      <HooksRow
        agents={[
          agent("claude", "Claude"),
          agent("gemini", "Gemini", { wired: false }),
          agent("cursor", "Cursor", { hooks: null }),
          agent("copilot", "Copilot", { hooks: undefined }),
        ]}
        onOpen={vi.fn()}
      />,
    );
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.queryByText(/Gemini|Cursor|Copilot/)).toBeNull();
  });

  it("says no agent is connected when none is, or when they are not read yet", () => {
    const { rerender } = render(<HooksRow agents={[]} onOpen={vi.fn()} />);
    expect(screen.getByText("No agent connected on this computer")).toBeTruthy();
    rerender(<HooksRow agents={null} onOpen={vi.fn()} />);
    expect(screen.getByText("No agent connected on this computer")).toBeTruthy();
    rerender(<HooksRow agents={[agent("gemini", "Gemini", { wired: false })]} onOpen={vi.fn()} />);
    expect(screen.getByText("No agent connected on this computer")).toBeTruthy();
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  it("opens Settings when a row is clicked", () => {
    const onOpen = vi.fn();
    render(<HooksRow agents={[agent("claude", "Claude")]} onOpen={onOpen} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("speaks Spanish when the app does", () => {
    adopt("es");
    render(
      <HooksRow
        agents={[agent("claude", "Claude", { hooks: { on: false, at: "x" } })]}
        onOpen={vi.fn()}
      />,
    );
    expect(screen.getByRole("button").textContent).toBe("Claude en este equipo");
  });
});

describe("the hooks row shows a dot, not a word", () => {
  const dotOf = (button: HTMLElement) => button.querySelector("[data-on]") as HTMLElement;

  it.each([
    ["en", { on: true, at: "x" }, true, "Claude on this computer: with hooks"],
    ["en", { on: false, at: "x" }, false, "Claude on this computer: no hooks"],
    ["en", { on: false, plugged: true, at: "x" }, true, "Claude on this computer: with hooks"],
    ["es", { on: true, at: "x" }, true, "Claude en este equipo: con hooks"],
    ["es", { on: false, at: "x" }, false, "Claude en este equipo: sin hooks"],
    ["es", { on: false, plugged: true, at: "x" }, true, "Claude en este equipo: con hooks"],
  ] as const)("in %s with %j: name, title and dot agree", (lang, hooks, on, said) => {
    adopt(lang);
    render(
      <HooksRow agents={[agent("claude", "Claude", { hooks: { ...hooks } })]} onOpen={vi.fn()} />,
    );
    const button = screen.getByRole("button", { name: said });
    const open =
      lang === "en"
        ? "Open in Settings › Agents › Connected"
        : "Abrir en Configuración › Agentes › Conectados";
    expect(button.getAttribute("title")).toBe(`${said} · ${open}`);
    expect(button.textContent).not.toMatch(/hooks/i);
    const dot = dotOf(button);
    expect(dot.getAttribute("aria-hidden")).toBe("true");
    expect(dot.getAttribute("data-on")).toBe(String(on));
    expect(dot.className).toContain(on ? "bg-hue-green" : "border-faint");
    expect(dot.className).not.toContain(on ? "border-faint" : "bg-hue-green");
  });
});

describe("tistyHookLines", () => {
  it("gives one line per connected agent with hooks, on when switched on or plugged", () => {
    expect(
      tistyHookLines([
        agent("claude", "Claude"),
        agent("codex", "Codex", { hooks: { on: false, at: "x" } }),
        agent("gemini", "Gemini", { hooks: { on: false, plugged: true, at: "x" } }),
        agent("cursor", "Cursor", { hooks: null }),
        agent("copilot", "Copilot", { wired: false }),
      ]),
    ).toEqual([
      { id: "claude", name: "Claude", on: true, byHand: false },
      { id: "codex", name: "Codex", on: false, byHand: true },
      { id: "gemini", name: "Gemini", on: true, byHand: false },
    ]);
  });

  it("gives none when the agents are not read yet", () => {
    expect(tistyHookLines(null)).toEqual([]);
    expect(tistyHookLines([])).toEqual([]);
  });
});

describe("the hooks row as a row of the tree", () => {
  it("passes what rowOf gives for each line, keyed by its agent", () => {
    const rowOf = vi.fn((key: string) => ({ "data-row": key, tabIndex: -1 }));
    render(
      <HooksRow
        agents={[agent("claude", "Claude"), agent("codex", "Codex")]}
        onOpen={vi.fn()}
        rowOf={rowOf}
      />,
    );
    const [first, second] = screen.getAllByRole("button");
    expect(first.getAttribute("data-row")).toBe("hooks:claude");
    expect(second.getAttribute("data-row")).toBe("hooks:codex");
    expect(first.tabIndex).toBe(-1);
    expect(rowOf).toHaveBeenCalledWith("hooks:claude");
  });

  it("is no row without rowOf", () => {
    render(<HooksRow agents={[agent("claude", "Claude")]} onOpen={vi.fn()} />);
    expect(screen.getByRole("button").hasAttribute("data-row")).toBe(false);
  });
});
