import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import {
  dragTo,
  fold,
  NARROWEST,
  nudge,
  RAIL,
  sideWidth,
  unwiden,
  useBreadth,
  WIDEST,
} from "../breadth";
import { toldAside } from "../coming";
import { t } from "../locales";
import Sidebar from "../ui/Sidebar";

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

describe("in a narrow window the sidebar gives way first", () => {
  const heard = new Set<() => void>();
  const plain = window.matchMedia;
  let narrow = false;

  const resize = (to: boolean) =>
    act(() => {
      narrow = to;
      for (const one of heard) one();
    });

  beforeEach(() => {
    window.matchMedia = ((query: string) => ({
      get matches() {
        return narrow && query.includes("1440px");
      },
      addEventListener: (_: string, one: () => void) => heard.add(one),
      removeEventListener: (_: string, one: () => void) => heard.delete(one),
    })) as unknown as typeof window.matchMedia;
  });

  afterEach(() => {
    act(() => {
      narrow = true;
      fold(true);
      narrow = false;
      unwiden();
    });
    window.matchMedia = plain;
    heard.clear();
  });

  it("folds to its icons as the window narrows, and opens again as it widens", () => {
    render(<Shown />);
    expect(width()).toBe("336");

    resize(true);
    expect(width()).toBe(String(RAIL));

    resize(false);
    expect(width()).toBe("336");
  });

  it("keeps open in a narrow window what was opened there, apart from the wide one", () => {
    render(<Shown />);
    resize(true);
    act(() => fold(false));
    expect(width()).toBe("336");

    resize(false);
    act(() => fold(true));
    expect(width()).toBe(String(RAIL));

    resize(true);
    expect(width()).toBe("336");
  });
});
