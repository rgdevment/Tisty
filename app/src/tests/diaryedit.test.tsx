import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { t } from "../locales";
import Docs from "../ui/Docs";

const BOOK = "a3f1-0010";
const MONTH = "a3f1-0011";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_opened")
      return Promise.resolve({ body: "# Noviembre\n\n## Domingo 15\n\nPaseo.\n", print: "p" });
    if (cmd === "diary_of")
      return Promise.resolve({ book: BOOK, page: MONTH, day: "Lunes 16", open: false });
    if (cmd === "diary_newest") return Promise.resolve(MONTH);
    return Promise.resolve(null);
  },
}));

vi.mock("../ui/Sheet", () => ({ default: () => <div data-testid="sheet" /> }));
vi.mock("../ui/Page", () => ({ default: () => <div data-testid="days" /> }));

const known: Filed[] = [
  {
    id: "01Z",
    file: BOOK,
    title: "Diario 2026",
    folder: null,
    archived: false,
    away: false,
    diary: true,
  },
  {
    id: "01N",
    file: MONTH,
    title: "Noviembre",
    folder: null,
    pageOf: "01Z",
    archived: false,
    away: false,
    diary: true,
  },
];

const shown = (visit: object) => (
  <Docs open={MONTH} visit={visit} known={known} onKept={vi.fn()} onError={vi.fn()} />
);

describe("a diary month written as a document", () => {
  it("heads this month with today as the core names it", async () => {
    render(shown({ named: "docs", doc: MONTH }));

    expect(await screen.findByText("Lunes 16")).toBeTruthy();
  });

  it("lasts only while it is the visit that asked for it", async () => {
    const { rerender } = render(shown({ named: "docs", doc: MONTH }));

    await userEvent.click(await screen.findByRole("button", { name: t("diaryEditMonth") }));

    expect(await screen.findByTestId("sheet")).toBeTruthy();
    expect(screen.queryByTestId("days")).toBeNull();
    expect(
      screen.queryByRole("button", { name: t("beside") }),
      "the diary has no side column",
    ).toBeNull();

    rerender(shown({ named: "docs", doc: MONTH }));

    expect(await screen.findByTestId("days")).toBeTruthy();
    expect(screen.queryByTestId("sheet")).toBeNull();
  });
});

describe("a diary year", () => {
  it("opens on its newest month and never shows the book's own page", async () => {
    const onDoc = vi.fn();
    render(
      <Docs
        open={BOOK}
        visit={{ named: "docs", doc: BOOK }}
        known={known}
        onDoc={onDoc}
        onKept={vi.fn()}
        onError={vi.fn()}
      />,
    );

    await waitFor(() => expect(onDoc).toHaveBeenCalledWith(MONTH));
    expect(screen.queryByTestId("days")).toBeNull();
    expect(screen.queryByTestId("sheet")).toBeNull();
  });
});
