import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Modal from "../ui/Modal";
import Fields from "../ui/Fields";
import type { Task } from "../core";

const watching = vi.hoisted(() => ({ tell: null as ((e: { payload: boolean }) => void) | null }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "main",
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
    onFocusChanged: (fn: (e: { payload: boolean }) => void) => {
      watching.tell = fn;
      return Promise.resolve(() => {});
    },
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

  it("starts with the focus inside it", () => {
    show();

    expect(document.activeElement).toBe(screen.getByRole("button", { name: /tray/i }));
  });

  it("announces itself as a dialog with its title", () => {
    show();

    expect(screen.getByRole("dialog", { name: "Close the window?" })).toBeTruthy();
  });

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

  it("ignores Escape where there is nothing to back out to", async () => {
    show();

    await userEvent.keyboard("{Escape}");

    expect(screen.getByRole("dialog")).toBeTruthy();
  });

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
  it("names the window buttons instead of reading their glyph", async () => {
    const { default: WindowChrome } = await import("../ui/WindowChrome");
    render(<WindowChrome />);

    expect(screen.getByRole("button", { name: "Minimise" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "─" })).toBeNull();
  });
});

describe("the window buttons sit where the system puts them", () => {
  const named = () =>
    screen.getAllByRole("button").map((one) => one.getAttribute("aria-label"));

  const chrome = async (agent: string) => {
    vi.resetModules();
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
    const { default: WindowChrome } = await import("../ui/WindowChrome");
    render(<WindowChrome />);
  };

  it("keeps closing on the right on Windows", async () => {
    await chrome("Mozilla/5.0 (Windows NT 10.0; Win64; x64)");

    expect(named()).toEqual(["Minimise", "Maximise", "Close"]);
  });

  it("puts them in the order macOS puts them", async () => {
    await chrome("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)");

    expect(named()).toEqual(["Close", "Minimise", "Maximise"]);
  });

  it("wears the three colours of the system on macOS", async () => {
    await chrome("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)");
    const lit = named().map(
      (name) => screen.getByRole("button", { name: name as string }).className,
    );

    expect(lit[0]).toContain("bg-[#ff5f57]");
    expect(lit[1]).toContain("bg-[#febc2e]");
    expect(lit[2]).toContain("bg-[#28c840]");
  });

  it("leaves Windows the thin glyphs that belong there", async () => {
    await chrome("Mozilla/5.0 (Windows NT 10.0; Win64; x64)");

    expect(screen.getByRole("button", { name: "Close" }).className).not.toContain("#ff5f57");
    expect(screen.getByRole("button", { name: "Close" }).textContent).toBe("✕");
  });

  it("goes grey when the window is not the one in front", async () => {
    await chrome("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)");
    await act(async () => {});
    const close = screen.getByRole("button", { name: "Close" });
    expect(close.classList.contains("bg-[#ff5f57]")).toBe(true);

    await act(async () => watching.tell?.({ payload: false }));

    expect(close.classList.contains("bg-[#ff5f57]")).toBe(false);
    expect(close.classList.contains("bg-line")).toBe(true);
    expect(close.classList.contains("group-hover:bg-[#ff5f57]")).toBe(true);
  });
});

describe("opening a task", () => {
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
