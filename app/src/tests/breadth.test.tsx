import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { dragTo, fold, RAIL, sideWidth, unwiden, useBreadth, WIDEST } from "../breadth";
import { t } from "../locales";
import Sidebar from "../ui/Sidebar";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const chosen = { named: "tasks" } as Parameters<typeof Sidebar>[0]["chosen"];

const bar = () =>
  render(
    <Sidebar
      lists={[]}
      papers={{ folders: [], docs: [] }}
      counts={{ tasks: 60 }}
      chosen={chosen}
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

describe("the sidebar can be folded and widened", () => {
  afterEach(() => act(() => unwiden()));

  it("keeps a width within its bounds, and folds when dragged past its narrowest", () => {
    render(<Shown />);
    expect(screen.getByRole("status").textContent).toBe("336");

    act(() => dragTo(900, true));
    expect(screen.getByRole("status").textContent).toBe(String(WIDEST));

    act(() => dragTo(90, true));
    expect(screen.getByRole("status").textContent).toBe(String(RAIL));

    act(() => fold(false));
    expect(screen.getByRole("status").textContent).toBe(String(WIDEST));
  });

  it("folds to icons that still say their names, and unfolds from the rail", () => {
    bar();
    act(() => fold(true));

    expect(screen.queryByText(t("tasks"))).toBeNull();
    expect(screen.getByRole("button", { name: `${t("tasks")} · 60` })).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: t("sideUnfold") }));
    expect(screen.getByText(t("tasks"))).toBeTruthy();
  });

  it("folds with the shortcut, but never while the person is writing", () => {
    bar();
    const field = document.createElement("textarea");
    document.body.append(field);

    fireEvent.keyDown(field, { key: "b", ctrlKey: true });
    expect(screen.getByText(t("tasks"))).toBeTruthy();

    fireEvent.keyDown(document.body, { key: "b", ctrlKey: true });
    expect(screen.queryByText(t("tasks"))).toBeNull();
    field.remove();
  });
});
