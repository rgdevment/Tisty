import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import {
  dragTo,
  fold,
  NARROWEST,
  nudge,
  RAIL,
  sideWidth,
  unwiden,
  useBreadth,
  useRoomFor,
  WIDEST,
} from "../breadth";
import { toldAside } from "../coming";
import { DOC_ASIDE_FROM, layoutOf } from "../layout";
import { t } from "../locales";
import Sidebar from "../ui/Sidebar";
import type { Chosen } from "../views";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const chosen = { named: "tasks" } as Parameters<typeof Sidebar>[0]["chosen"];

const bar = (at = chosen) =>
  render(
    <Sidebar
      lists={[]}
      papers={{ folders: [], docs: [] }}
      counts={{ tasks: 60 }}
      chosen={at}
      onChoose={vi.fn()}
      onFile={vi.fn()}
      onHere={vi.fn()}
      onMove={vi.fn()}
      onFolderMenu={vi.fn()}
      onDocMenu={vi.fn()}
      onDocsMenu={vi.fn()}
      onHereMenu={vi.fn()}
    />,
  );

function Shown() {
  const breadth = useBreadth();
  return <output>{sideWidth(breadth, false)}</output>;
}

const width = () => screen.getByRole("status").textContent;

function Roomed({ need }: { need: number }) {
  useRoomFor(need);
  return <output>{sideWidth(useBreadth(), false)}</output>;
}

describe("the sidebar can be folded and widened", () => {
  beforeAll(() => {
    if (!Element.prototype.setPointerCapture) {
      Element.prototype.setPointerCapture = vi.fn();
      Element.prototype.releasePointerCapture = vi.fn();
    }
  });
  afterEach(() => act(() => unwiden()));

  it("keeps a width within its bounds, and folds when dragged past its narrowest", () => {
    render(<Shown />);
    expect(width()).toBe("336");

    act(() => dragTo(900));
    expect(width()).toBe(String(WIDEST));

    act(() => dragTo(90));
    expect(width()).toBe(String(RAIL));

    act(() => fold(false));
    expect(width()).toBe(String(WIDEST));
  });

  it("narrows by keys until it folds, and a key never unfolds it the wrong way", () => {
    render(<Shown />);

    act(() => nudge(NARROWEST, -16));
    expect(width()).toBe(String(RAIL));

    act(() => nudge(RAIL, -16));
    expect(width()).toBe(String(RAIL));

    act(() => nudge(RAIL, 16));
    expect(width()).toBe("336");
  });

  it("keeps the usual width when its edge is only clicked", () => {
    render(<Shown />);
    bar();
    const edge = screen.getByRole("separator");

    fireEvent.pointerDown(edge, { button: 0, clientX: 336, pointerId: 1 });
    fireEvent.pointerUp(edge, { clientX: 337, pointerId: 1 });

    expect(width()).toBe("336");
    expect(localStorage.getItem("tisty.breadth")).not.toContain("336");
  });

  it("shows the rail a dot when something waits, and the dot unfolds it", () => {
    bar();
    act(() => fold(true));
    expect(screen.queryByRole("button", { name: t("sideNotices") })).toBeNull();

    act(() => toldAside("a round waits for a choice"));
    fireEvent.click(screen.getByRole("button", { name: t("sideNotices") }));

    expect(screen.getByText(t("tasks"))).toBeTruthy();
    act(() => toldAside(null));
  });

  it("folds to icons that still say their names, and unfolds from the rail", () => {
    bar();
    act(() => fold(true));

    expect(screen.queryByText(t("tasks"))).toBeNull();
    expect(screen.getByRole("button", { name: `${t("tasks")} · 60` })).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: t("sideUnfold") }));
    expect(screen.getByText(t("tasks"))).toBeTruthy();
  });

  it("folds with the shortcut, but never while the person writes or stands in settings", () => {
    const { unmount } = bar();
    const field = document.createElement("textarea");
    document.body.append(field);

    fireEvent.keyDown(field, { key: "b", ctrlKey: true });
    expect(screen.getByText(t("tasks"))).toBeTruthy();

    fireEvent.keyDown(document.body, { key: "b", ctrlKey: true });
    expect(screen.queryByText(t("tasks"))).toBeNull();
    field.remove();
    unmount();

    act(() => fold(false));
    render(<Shown />);
    bar({ named: "keeping" } as typeof chosen);
    fireEvent.keyDown(document.body, { key: "b", ctrlKey: true });
    expect(width()).toBe("336");
  });
});

describe("the sidebar gives way to the column on the right", () => {
  const across = window.innerWidth;

  const resize = (to: number) =>
    act(() => {
      Object.defineProperty(window, "innerWidth", {
        value: to,
        configurable: true,
        writable: true,
      });
      window.dispatchEvent(new Event("resize"));
    });

  afterEach(() => {
    resize(across);
    render(<Roomed need={0} />);
    act(() => unwiden());
  });

  it("folds to its icons only where that makes the column fit", () => {
    resize(1100);
    const { rerender } = render(<Roomed need={884} />);
    expect(width()).toBe(String(RAIL));

    resize(1300);
    expect(width()).toBe("336");

    resize(900);
    expect(width()).toBe("336");

    resize(1100);
    rerender(<Roomed need={0} />);
    expect(width()).toBe("336");
  });

  it("keeps open what was opened by hand, until the view or the window changes", () => {
    resize(1100);
    const { rerender } = render(<Roomed need={884} />);
    act(() => fold(false));
    expect(width()).toBe("336");

    rerender(<Roomed need={964} />);
    expect(width()).toBe(String(RAIL));

    act(() => fold(false));
    resize(1400);
    resize(1100);
    expect(width()).toBe(String(RAIL));
  });

  it("forgets a hand open once a narrower sidebar no longer crowds the column", () => {
    resize(1190);
    const { rerender } = render(<Roomed need={884} />);
    act(() => fold(false));
    expect(width()).toBe("336");

    act(() => dragTo(NARROWEST));
    rerender(<Roomed need={884} />);
    expect(width()).toBe(String(NARROWEST));

    act(() => dragTo(400));
    expect(width()).toBe(String(RAIL));
  });

  it("leaves the choice made where there is room as it was", () => {
    resize(1100);
    render(<Roomed need={884} />);
    act(() => fold(false));
    resize(1600);
    expect(width()).toBe("336");
  });
});

describe("what each view needs on its right", () => {
  const room = (chosen: Chosen, open = false) =>
    layoutOf({ chosen, open, mode: "columns", tight: false, calm: true }).room;

  it("asks room for the day beside the tasks, more with a task open, and for a document's column", () => {
    expect(room({ named: "tasks" })).toBe(884);
    expect(room({ named: "tasks" }, true)).toBe(1536);
    expect(room({ named: "docs", doc: "a.md" })).toBe(DOC_ASIDE_FROM);
    expect(room({ named: "docs" })).toBe(0);
    const shut = layoutOf({
      chosen: { named: "docs", doc: "a.md" },
      open: false,
      mode: "columns",
      tight: false,
      calm: true,
      asideShut: true,
    });
    expect(shut.room).toBe(0);
    expect(room({ named: "spread" })).toBe(0);
  });
});
