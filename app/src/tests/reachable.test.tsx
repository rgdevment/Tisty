import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Modal from "../ui/Modal";
import Fields from "../ui/Fields";
import type { Task } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "main",
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

const task = {
  id: "01T",
  title: "sacar la basura",
  status: "open",
  priority: 4,
  order: "a0",
  tags: [],
  reminders: [],
} as unknown as Task;

describe("a modal holds the keyboard while it is up", () => {
  const show = (onClose?: () => void) =>
    render(
      <Modal title="Close the window?" onClose={onClose}>
        <button type="button">Leave it in the tray</button>
        <button type="button">Quit</button>
      </Modal>,
    );

  /// Otherwise the first Tab lands behind the veil, on controls nobody can see.
  it("starts with the focus inside it", () => {
    show();

    expect(document.activeElement).toBe(screen.getByRole("button", { name: /tray/i }));
  });

  it("announces itself as a dialog with its title", () => {
    show();

    expect(screen.getByRole("dialog", { name: "Close the window?" })).toBeTruthy();
  });

  /// While a modal is up nothing behind it is reachable, by mouse or otherwise.
  it("wraps around instead of tabbing out of the back", async () => {
    show();

    await userEvent.tab();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Quit" }));

    await userEvent.tab();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: /tray/i }));
  });

  it("wraps the other way with Shift", async () => {
    show();

    await userEvent.tab({ shift: true });

    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Quit" }));
  });

  it("backs out on Escape when backing out is an answer", async () => {
    const closed = vi.fn();
    show(closed);

    await userEvent.keyboard("{Escape}");

    expect(closed).toHaveBeenCalled();
  });

  /// The welcome has to be answered before anything else can be reached.
  it("ignores Escape where there is nothing to back out to", async () => {
    show();

    await userEvent.keyboard("{Escape}");

    expect(screen.getByRole("dialog")).toBeTruthy();
  });

  /// A dialog that drops focus on the body sends the next Tab to the top of
  /// the window, nowhere near what the person was doing.
  it("hands the focus back where it came from", async () => {
    render(
      <>
        <button type="button">the row</button>
        <div id="slot" />
      </>,
    );
    const came = screen.getByRole("button", { name: "the row" });
    came.focus();

    const dialog = render(
      <Modal title="Close the window?" onClose={() => {}}>
        <button type="button">Quit</button>
      </Modal>,
    );
    dialog.unmount();

    expect(document.activeElement).toBe(came);
  });
});

describe("the field sheets in the detail", () => {
  const show = () => render(<Fields task={task} lists={[]} known={[]} onPatch={() => {}} />);

  it("says whether it is open", async () => {
    show();
    const chip = screen.getByRole("button", { name: /priority/i });

    expect(chip.getAttribute("aria-expanded")).toBe("false");

    await userEvent.click(chip);

    expect(chip.getAttribute("aria-expanded")).toBe("true");
  });

  /// The catcher behind it is for the mouse only.
  it("closes on Escape", async () => {
    show();

    await userEvent.click(screen.getByRole("button", { name: /priority/i }));
    expect(screen.getByRole("button", { name: /^High/ })).toBeTruthy();

    await userEvent.keyboard("{Escape}");

    expect(screen.queryByRole("button", { name: /^High/ })).toBeNull();
  });

  it("puts the focus on the first choice, not behind the sheet", async () => {
    show();

    await userEvent.click(screen.getByRole("button", { name: /priority/i }));

    expect((document.activeElement as HTMLElement).textContent).toMatch(/High/);
  });
});

describe("what a screen reader is told", () => {
  /// The buttons are glyphs: without a label a reader announced «─» and «□».
  it("names the window buttons instead of reading their glyph", async () => {
    const { default: WindowChrome } = await import("../ui/WindowChrome");
    render(<WindowChrome />);

    expect(screen.getByRole("button", { name: "Minimise" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "─" })).toBeNull();
  });
});

describe("opening a task", () => {
  /// It moved the eye and left the keyboard back in the list, so the next Tab
  /// went nowhere near what had just been opened.
  it("takes the focus with it", async () => {
    const { default: Detail } = await import("../ui/Detail");
    render(
      <Detail
        task={task}
        lists={[]}
        known={[]}
        expanded={false}
        onExpand={() => {}}
        onCollapse={() => {}}
        onPatch={() => {}}
        onStep={() => {}}
        onMark={() => {}}
        onDropStep={() => {}}
        onLog={() => {}}
        onDiscard={() => {}}
        onReopen={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );

    expect((document.activeElement as HTMLElement).tagName).toBe("ASIDE");
  });
});

describe("the / menu says which row is live", () => {
  /// Arrow keys move a highlight the eye can follow and a reader could not:
  /// the focus never leaves the text field, so without this nothing is said.
  it("points at the highlighted option, and moves the pointer with it", async () => {
    const { default: SlashMenu } = await import("../ui/SlashMenu");
    render(
      <SlashMenu
        from={null}
        query=""
        lists={[]}
        tags={[]}
        onDate={() => {}}
        onInsert={() => {}}
        onClose={() => {}}
      />,
    );

    const box = screen.getByRole("listbox");
    const first = box.getAttribute("aria-activedescendant");
    expect(first).toBeTruthy();
    expect(document.getElementById(first as string)?.getAttribute("aria-selected")).toBe("true");

    await userEvent.keyboard("{ArrowDown}");

    const second = box.getAttribute("aria-activedescendant");
    expect(second).not.toBe(first);
    expect(document.getElementById(second as string)?.getAttribute("aria-selected")).toBe("true");
  });
});
