import { fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed, Papers } from "../core";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => {
  const open = { makes: ["doc"], pages: true, moves: true, copies: true };
  const ruling = {
    plain: open,
    agents: open,
    ideas: { makes: [], pages: false, moves: false, copies: false },
    diary: { makes: [], pages: false, moves: false, copies: true },
  };
  return {
    invoke: (cmd: string) =>
      Promise.resolve(cmd === "places_rules" ? ruling : cmd === "icons" ? [] : null),
  };
});

const paper = (id: string, title: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

describe("the tree on its own", () => {
  it("asks the core what each place takes, so Ideas and the diary are no drop targets", async () => {
    document.elementFromPoint = () => null;
    const board = paper("i1", "Ideas", { file: "ideas-file" });
    const book = paper("b1", "Diario 2026", { diary: true });
    const papers: Papers = { folders: [], docs: [board, book, paper("d1", "Uno")] };
    const { container } = render(
      <Tree
        papers={papers}
        unfolded
        pinned={board.file}
        onDiary={vi.fn()}
        onOpen={vi.fn()}
        onFile={vi.fn()}
      />,
    );

    await waitFor(() => expect(container.querySelector('[data-drop="i1"]')).toBeNull());
    expect(container.querySelector('[data-drop="b1"]')).toBeNull();
    expect(container.querySelector('[data-drop="d1"]')).not.toBeNull();
    const ideas = container.querySelector<HTMLElement>('[data-row="i1"]') as HTMLElement;
    fireEvent.pointerDown(ideas, { button: 0, clientX: 0, clientY: 0 });
    fireEvent.pointerMove(window, { clientX: 0, clientY: 40 });
    expect(ideas.className.includes("opacity-45")).toBe(false);
    fireEvent.pointerUp(window, { clientX: 0, clientY: 40 });
  });
});
