import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Folded, Papers } from "../core";
import { BRAINSTORM, DIARY, hide, hideChoice, isHidden, show, showAll } from "../hiding";
import SideRail from "../ui/SideRail";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve([]) }));

const folder = (id: string, name: string, parent: string | null = null): Folded => ({
  id,
  name,
  parent,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
});

const papers: Papers = {
  folders: [folder("F1", "agents"), folder("F2", "claude", "F1"), folder("F3", "trabajo")],
  docs: [
    { id: "D1", file: "f1", title: "Reglas", folder: "F2", archived: false, away: false },
    { id: "D2", file: "f2", title: "Ideas", folder: "F3", archived: false, away: false },
    {
      id: "D3",
      file: "f3",
      title: "2026",
      folder: "F3",
      archived: false,
      away: false,
      diary: true,
    },
  ],
};

const drawn = (props: Partial<React.ComponentProps<typeof Tree>> = {}) =>
  render(
    <Tree
      papers={papers}
      pinned="f2"
      onOpen={vi.fn()}
      onFile={vi.fn()}
      onDiary={vi.fn()}
      onHere={vi.fn()}
      {...props}
    />,
  );

const tree = () => screen.getByRole("list", { name: "Documents" });
const rowStops = () =>
  Array.from(tree().querySelectorAll<HTMLElement>("[data-row]")).filter(
    (one) => one.tabIndex === 0,
  );
const hiddenList = () => screen.queryByRole("list", { name: "Hidden" });

describe("hiding rows from the sidebar", () => {
  beforeEach(() => showAll());

  it("draws no section for hidden rows, even with rows hidden", () => {
    hide("F1");
    hide(DIARY);
    drawn();

    expect(hiddenList()).toBeNull();
    expect(screen.queryByRole("button", { name: "Show hidden" })).toBeNull();
  });

  it("takes a hidden folder and all it holds out of the sidebar without a trace", () => {
    hide("F1");
    drawn();

    expect(within(tree()).queryByRole("button", { name: "agents" })).toBeNull();
    expect(within(tree()).getByRole("button", { name: "trabajo" })).toBeTruthy();
    expect(screen.queryByText("agents")).toBeNull();
  });

  it("does not call a folder empty when all it holds is a hidden folder", () => {
    hide("F2");
    drawn();
    fireEvent.click(screen.getByRole("button", { name: "Open agents" }));

    expect(within(tree()).queryByText("empty")).toBeNull();
  });

  it("keeps a hidden diary and brainstorm out of the tree, and out of the folders they sit in", () => {
    hide(DIARY);
    hide(BRAINSTORM);
    drawn();
    fireEvent.click(screen.getByRole("button", { name: "Open trabajo" }));

    expect(within(tree()).queryByRole("button", { name: "Diary" })).toBeNull();
    expect(within(tree()).queryByRole("button", { name: "Ideas" })).toBeNull();
    expect(within(tree()).queryByRole("button", { name: "2026" })).toBeNull();
    expect(hiddenList()).toBeNull();
  });

  it("leaves the tab stop on a drawn row when the rows ahead of it are hidden", () => {
    hide(BRAINSTORM);
    hide(DIARY);
    hide("F1");
    drawn();

    const stops = rowStops();
    expect(stops.map((one) => one.dataset.row)).toEqual(["F3"]);
  });

  it("gives the tab stop back to a drawn row when the row holding it is hidden", () => {
    drawn();
    act(() => within(tree()).getByRole("button", { name: "agents" }).focus());

    act(() => hide("F1"));

    const stops = rowStops();
    expect(stops).toHaveLength(1);
  });

  it("shows what is hidden while the sidebar is sifted, so a match is never behind a hidden row", () => {
    hide("F1");
    drawn({ unfolded: true });

    expect(within(tree()).getByRole("button", { name: "Reglas" })).toBeTruthy();
    expect(hiddenList()).toBeNull();
  });

  it("opens the diary's menu from a right click and from Shift+F10", () => {
    const onDiaryMenu = vi.fn();
    drawn({ onDiaryMenu });
    const diary = within(tree()).getByRole("button", { name: "Diary" });

    fireEvent.contextMenu(diary);
    fireEvent.keyDown(diary, { key: "F10", shiftKey: true });

    expect(onDiaryMenu).toHaveBeenCalledTimes(2);
  });

  it("keeps a hidden folder inside the archived one that holds it, where it can still be reached", () => {
    hide("F2");
    const shelved: Papers = {
      folders: [
        { ...folder("F1", "agents"), archived: true, away: true },
        { ...folder("F2", "claude", "F1"), away: true },
      ],
      docs: [],
    };
    render(<Tree papers={shelved} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Archived" }));
    fireEvent.click(screen.getByRole("button", { name: "Open agents" }));

    expect(screen.getByRole("button", { name: "claude" })).toBeTruthy();
    expect(hiddenList()).toBeNull();
  });

  it("leaves a hidden folder out of the folded sidebar too", () => {
    hide("F1");
    render(
      <SideRail
        lists={[]}
        papers={papers}
        counts={{}}
        chosen={{ named: "tasks" }}
        onChoose={vi.fn()}
        onReveal={vi.fn()}
      />,
    );

    expect(screen.queryByRole("button", { name: /agents/ })).toBeNull();
    expect(screen.getByRole("button", { name: /trabajo/ })).toBeTruthy();
  });

  it("turns the menu choice into its opposite once the row is hidden", () => {
    expect(hideChoice("F3").label).toBe("Hide from the sidebar");
    hideChoice("F3").onPick?.();

    expect(isHidden("F3")).toBe(true);
    expect(hideChoice("F3").label).toBe("Show in the sidebar");
  });

  it("shows a row hidden under any of its keys, and showing takes every key away", () => {
    hide("old-id");
    const choice = hideChoice(["agents", "old-id"]);
    expect(choice.label).toBe("Show in the sidebar");
    choice.onPick?.();
    expect(isHidden("old-id")).toBe(false);

    hideChoice(["agents", "old-id"]).onPick?.();
    expect(isHidden("agents")).toBe(true);
    expect(isHidden("old-id")).toBe(false);
    show("agents");
    expect(isHidden("agents")).toBe(false);
  });
});
