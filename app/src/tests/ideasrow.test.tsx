import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Papers } from "../core";
import { BRAINSTORM, hide } from "../hiding";
import { t } from "../locales";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const sheet = {
  id: "01I",
  file: "a3f1-0002",
  title: "Lluvia de ideas",
  folder: null,
  archived: false,
  away: false,
};
const empty: Papers = { folders: [], docs: [] };
const ideas = () => screen.queryAllByRole("button", { name: t("brainstorm") });

beforeEach(() => localStorage.clear());

describe("Ideas in the library before any idea is written", () => {
  it("stands first with its own mark, holds the tab stop, and opens on a press", async () => {
    const onIdeas = vi.fn();
    render(
      <Tree papers={empty} onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} onIdeas={onIdeas} />,
    );

    const [row] = ideas();
    expect(ideas()).toHaveLength(1);
    expect(row.tabIndex).toBe(0);
    expect(row.closest("li")?.querySelector("svg")?.getAttribute("class")).toContain(
      "brainstorm-mark",
    );
    expect(
      row.compareDocumentPosition(screen.getByRole("button", { name: t("diary") })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    await userEvent.setup().click(row);
    expect(onIdeas).toHaveBeenCalledTimes(1);
  });

  it("offers its menu to the pointer and to the keys, so it can be hidden before it holds anything", async () => {
    const onIdeasMenu = vi.fn();
    render(
      <Tree
        papers={empty}
        onOpen={vi.fn()}
        onFile={vi.fn()}
        onIdeas={vi.fn()}
        onIdeasMenu={onIdeasMenu}
      />,
    );
    const user = userEvent.setup();
    const [row] = ideas();

    await user.pointer({ keys: "[MouseRight]", target: row });
    row.focus();
    await user.keyboard("{Shift>}{F10}{/Shift}");

    expect(onIdeasMenu).toHaveBeenCalledTimes(2);
    expect(row.getAttribute("aria-keyshortcuts")).toBe("Shift+F10");
  });

  it("hands the arrows on to the Diary below it and back", async () => {
    render(
      <Tree papers={empty} onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} onIdeas={vi.fn()} />,
    );
    const user = userEvent.setup();
    const [row] = ideas();
    row.focus();

    await user.keyboard("{ArrowDown}");
    expect(document.activeElement).toBe(screen.getByRole("button", { name: t("diary") }));
    await user.keyboard("{ArrowUp}");
    expect(document.activeElement).toBe(row);
  });

  it("gives way to the sheet once there is one", () => {
    const opened = vi.fn();
    render(
      <Tree
        papers={{ folders: [], docs: [sheet] }}
        pinned={sheet.file}
        onOpen={opened}
        onFile={vi.fn()}
        onIdeas={vi.fn()}
      />,
    );

    expect(ideas()).toHaveLength(1);
    expect(ideas()[0].getAttribute("data-row")).toBe(sheet.id);
  });

  it("stays away when the person hid Ideas, while a search runs, or where nothing opens it", () => {
    const tree = (more: object) => (
      <Tree papers={empty} onOpen={vi.fn()} onFile={vi.fn()} {...more} />
    );
    const { rerender } = render(tree({}));
    expect(ideas()).toHaveLength(0);

    rerender(tree({ onIdeas: vi.fn(), unfolded: true }));
    expect(ideas()).toHaveLength(0);

    hide(BRAINSTORM);
    rerender(tree({ onIdeas: vi.fn() }));
    expect(ideas()).toHaveLength(0);
  });
});
