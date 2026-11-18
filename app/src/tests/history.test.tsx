import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import History from "../ui/History";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { body?: string }) =>
    Promise.resolve(cmd === "note_drawn" ? `<p>${args.body ?? ""}</p>` : null),
}));

const task = (log: { id: string; body: string; by?: string; replaces?: string }[]) =>
  ({
    id: "01A",
    title: "write the report",
    status: "open",
    log: log.map((one) => ({ ...one, at: "2026-10-18 09:00:00", tz: "America/Santiago" })),
  }) as unknown as Task;

describe("what happened to a task", () => {
  it("opens under its words, counting only the entries that say something", async () => {
    render(
      <History
        task={task([
          { id: "1", body: "spoke to accounting" },
          { id: "2", body: "  " },
        ])}
      />,
    );

    const fold = screen.getByText(`${t("history")} · 1`).closest("details");
    expect(fold?.open).toBe(true);
    expect(await screen.findByText("spoke to accounting")).toBeTruthy();
  });

  it("keeps a note another one replaced, marked as such, and leaves the rest unmarked", async () => {
    render(
      <History
        task={task([
          { id: "1", body: "chose CSV" },
          { id: "2", body: "chose XLSX after all", replaces: "1" },
        ])}
      />,
    );

    expect(screen.getByText(`${t("history")} · 2`)).toBeTruthy();
    const old = (await screen.findByText("chose CSV")).closest("li");
    const now = (await screen.findByText("chose XLSX after all")).closest("li");
    expect(old?.textContent).toContain(t("historyReplaced"));
    expect(now?.textContent).not.toContain(t("historyReplaced"));
  });

  it("is not drawn at all while nothing has happened", () => {
    const { container } = render(<History task={task([{ id: "1", body: "" }])} />);

    expect(container.firstChild).toBeNull();
  });
});
