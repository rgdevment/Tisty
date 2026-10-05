import { act, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { heard } from "../coming";
import { t } from "../locales";
import Coming, { SHOWN_AFTER, SLOW_AFTER } from "../ui/Coming";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

afterEach(() => act(() => heard(null)));

describe("what is coming from the folder", () => {
  it("stays out of the way of a round that ends quickly", () => {
    render(<Coming />);
    act(() => heard({ stage: "papers", done: 3, whole: 10 }));

    expect(screen.queryByRole("status")).toBeNull();
  });

  it("says the stage and how far it got once a round runs long", () => {
    render(<Coming />);
    act(() => heard({ stage: "papers", done: 312, whole: 1040 }, Date.now() - SHOWN_AFTER));
    act(() => vi.advanceTimersByTime(1_000));

    expect(screen.getByRole("status").textContent).toContain(t("comingPapers"));
    expect(screen.getByRole("status").textContent).toMatch(/312.*1.040/);
    expect(screen.queryByText(t("comingSlow"))).toBeNull();
  });

  it("says the cloud is slow instead of keeping quiet about it", () => {
    render(<Coming />);
    act(() => heard({ stage: "attachments", done: 4, whole: 9 }, Date.now() - SLOW_AFTER));
    act(() => vi.advanceTimersByTime(1_000));

    expect(screen.getByText(t("comingSlow"))).toBeTruthy();
  });

  it("keeps the start of a round as the stages move on", () => {
    render(<Coming />);
    act(() => heard({ stage: "log", done: 0, whole: 2 }, Date.now() - SHOWN_AFTER));
    act(() => heard({ stage: "papers", done: 0, whole: 5 }));
    act(() => vi.advanceTimersByTime(1_000));

    expect(screen.getByRole("status").textContent).toContain(t("comingPapers"));
  });

  it("goes away when the round is over", () => {
    render(<Coming />);
    act(() => heard({ stage: "log", done: 1, whole: 2 }, Date.now() - SHOWN_AFTER));
    act(() => vi.advanceTimersByTime(1_000));
    act(() => heard(null));

    expect(screen.queryByRole("status")).toBeNull();
  });

  it("tells an empty list its history is on its way, not that there is nothing", () => {
    render(<TaskList tasks={[]} lists={[]} title="Hoy" onSelect={vi.fn()} />);
    expect(screen.getByText(t("nothingOpen"))).toBeTruthy();

    act(() => heard({ stage: "log", done: 0, whole: 3 }));

    expect(screen.getByText(t("comingHistory"))).toBeTruthy();
    expect(screen.queryByText(t("nothingOpen"))).toBeNull();

    act(() => heard({ stage: "papers", done: 0, whole: 3 }));

    expect(screen.getByText(t("nothingOpen"))).toBeTruthy();
  });
});
