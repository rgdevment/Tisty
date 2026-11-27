import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type { Filed, Papers } from "../core";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve([]) }));

const doc = (id: string, title: string, folder: string | null = null): Filed => ({
  id,
  file: `f-${id}`,
  title,
  folder,
  archived: false,
  away: false,
});

const FOLDER = {
  id: "F",
  name: "Casa",
  parent: null,
  icon: null,
  holds: 0,
  archived: false,
  away: false,
};

const LIST = [doc("a", "Alfa"), doc("b", "Beta"), doc("c", "Gama"), doc("d", "Delta")];

const seen = { picked: new Set<string>() };

function Holder({ papers }: { papers: Papers }) {
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  seen.picked = new Set(picked);
  return (
    <Tree
      papers={papers}
      onOpen={() => {}}
      onFile={() => {}}
      unfolded
      picked={picked}
      onPicked={(ids) => setPicked(new Set(ids))}
    />
  );
}

const row = (id: string) => document.querySelector(`[data-row="${id}"]`) as HTMLElement;
const focus = (id: string) => act(() => row(id).focus());
const shiftDown = (on: HTMLElement) => fireEvent.keyDown(on, { key: "ArrowDown", shiftKey: true });
const shiftUp = (on: HTMLElement) => fireEvent.keyDown(on, { key: "ArrowUp", shiftKey: true });
const ids = () => [...seen.picked].sort();

describe("picking documents with the keyboard in the tree", () => {
  it("picks from the focused document to the one it lands on, and moves the focus", () => {
    render(<Holder papers={{ folders: [], docs: LIST }} />);
    focus("a");

    shiftDown(row("a"));
    expect(ids()).toEqual(["a", "b"]);
    expect(document.activeElement).toBe(row("b"));

    shiftDown(row("b"));
    expect(ids()).toEqual(["a", "b", "c"]);
  });

  it("shrinks the run with Shift and the opposite arrow", () => {
    render(<Holder papers={{ folders: [], docs: LIST }} />);
    focus("a");
    shiftDown(row("a"));
    shiftDown(row("b"));

    shiftUp(row("c"));

    expect(ids()).toEqual(["a", "b"]);
    expect(document.activeElement).toBe(row("b"));
  });

  it("skips the rows that are not documents when it counts the run", () => {
    const papers: Papers = { folders: [FOLDER], docs: [...LIST.slice(0, 2), doc("e", "Eco", "F")] };
    render(<Holder papers={papers} />);
    const rows = [...document.querySelectorAll<HTMLElement>("[data-row]")];
    focus("e");
    fireEvent.click(row("e"), { ctrlKey: true });

    for (let step = 0; step < rows.length; step += 1) {
      shiftDown(document.activeElement as HTMLElement);
    }

    expect(ids()).toEqual(["a", "b", "e"]);
  });

  it("folding a folder lets go of what was picked", () => {
    render(<Holder papers={{ folders: [FOLDER], docs: [...LIST, doc("e", "Eco", "F")] }} />);
    focus("a");
    shiftDown(row("a"));
    expect(ids()).toEqual(["a", "b"]);

    fireEvent.click(screen.getByRole("button", { name: /^(Open|Close|Fold|Unfold) Casa/ }));

    expect(ids()).toEqual([]);
  });
});
