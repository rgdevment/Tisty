import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Papers } from "../core";
import { t } from "../locales";
import { IDEA_AT_MOST } from "../notes";
import DiaryComposer from "../ui/DiaryComposer";
import DiaryHead from "../ui/DiaryHead";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  open: false,
  holding: false,
}));

const TODAY = { book: "a3f1-0010", page: "a3f1-0011" };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "diary_of") return Promise.resolve({ book: TODAY.book, open: ipc.open });
    if (cmd === "diary_write" && ipc.holding) return new Promise(() => {});
    if (cmd === "diary_write" || cmd === "diary_stick") return Promise.resolve(TODAY);
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.calls = [];
  ipc.open = false;
  ipc.holding = false;
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const papers: Papers = {
  folders: [],
  docs: [
    { id: "01S", file: "a3f1-0001", title: "Suelto", folder: null, archived: false, away: false },
    {
      id: "01Y",
      file: "a3f1-0009",
      title: "Diario 2025",
      folder: null,
      archived: false,
      away: false,
      diary: true,
    },
    {
      id: "01Z",
      file: TODAY.book,
      title: "Diario 2026",
      folder: null,
      archived: false,
      away: false,
      diary: true,
    },
    {
      id: "01N",
      file: TODAY.page,
      title: "Noviembre",
      folder: null,
      pageOf: "01Z",
      archived: false,
      away: false,
      diary: true,
    },
  ],
};

describe("the diary in the library", () => {
  it("opens today from its own row and keeps its years and months under it", async () => {
    const onDiary = vi.fn();
    render(
      <Tree
        papers={papers}
        open={TODAY.page}
        onOpen={vi.fn()}
        onFile={vi.fn()}
        onDiary={onDiary}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: t("diary") }));

    expect(onDiary).toHaveBeenCalledOnce();
    const rows = Array.from(
      document.querySelectorAll<HTMLElement>("#pages-diary [data-row]"),
      (one) => one.getAttribute("aria-label"),
    );
    expect(rows).toEqual(["Diario 2026", "Noviembre", "Diario 2025"]);
  });

  it("never draws a year twice, under the diary and loose", () => {
    render(<Tree papers={papers} unfolded onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} />);

    const year = screen.getAllByRole("button", { name: "Diario 2026" });
    expect(year).toHaveLength(1);
    expect(year[0]?.closest("#pages-diary")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Suelto" }).closest("#pages-diary")).toBeNull();
  });

  it("puts the newest year on top whatever language each book is named in", () => {
    const years: Papers = {
      folders: [],
      docs: [
        {
          id: "01P",
          file: "a3f1-0021",
          title: "Diary 2026",
          folder: null,
          archived: false,
          away: false,
          diary: true,
        },
        {
          id: "01Q",
          file: "a3f1-0022",
          title: "Diario 2027",
          folder: null,
          archived: false,
          away: false,
          diary: true,
        },
      ],
    };
    render(<Tree papers={years} unfolded onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} />);

    const rows = Array.from(
      document.querySelectorAll<HTMLElement>("#pages-diary [data-row]"),
      (one) => one.getAttribute("aria-label"),
    );
    expect(rows).toEqual(["Diario 2027", "Diary 2026"]);
  });

  it("keeps the tab stop on the diary's own row when only diary books are there", () => {
    const only: Papers = { folders: [], docs: papers.docs.filter((one) => one.diary) };
    render(<Tree papers={only} onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} />);

    expect(screen.getByRole("button", { name: t("diary") }).tabIndex).toBe(0);
  });

  it("wears its own colour on its row, with books or without", () => {
    const empty: Papers = { folders: [], docs: [] };
    const { unmount } = render(
      <Tree papers={empty} onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} />,
    );
    const mark = () =>
      screen
        .getByRole("button", { name: t("diary") })
        .closest("li")
        ?.querySelector("svg");
    expect(mark()?.getAttribute("class")).toContain("text-hue-purple");
    unmount();

    render(<Tree papers={papers} unfolded onOpen={vi.fn()} onFile={vi.fn()} onDiary={vi.fn()} />);
    expect(mark()?.getAttribute("class")).toContain("text-hue-purple");
  });

  it("shows the books where they sit when nothing gathers them", () => {
    render(<Tree papers={papers} unfolded onOpen={vi.fn()} onFile={vi.fn()} />);

    expect(screen.queryByRole("button", { name: t("diary") })).toBeNull();
    expect(screen.getByRole("button", { name: "Diario 2026" })).toBeTruthy();
  });
});

describe("today's box", () => {
  const box = (onWritten = vi.fn(), onBefore = vi.fn(() => Promise.resolve())) =>
    render(<DiaryComposer onBefore={onBefore} onWritten={onWritten} onError={vi.fn()} />);

  it("writes what is said into today and hands back where it landed", async () => {
    const onWritten = vi.fn();
    const onBefore = vi.fn(() => Promise.resolve());
    box(onWritten, onBefore);

    await userEvent.type(
      screen.getByRole("textbox", { name: t("diaryWriting") }),
      "Paseo con Rina",
    );
    await userEvent.keyboard("{Control>}{Enter}{/Control}");

    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(onBefore).toHaveBeenCalledOnce();
    expect(asked("diary_write")).toEqual([
      { cmd: "diary_write", args: { text: "Paseo con Rina" } },
    ]);
    expect((screen.getByRole("textbox") as HTMLTextAreaElement).value).toBe("");
  });

  it("writes once however many times the words are kept while the first is on its way", async () => {
    ipc.holding = true;
    box();

    await userEvent.type(screen.getByRole("textbox", { name: t("diaryWriting") }), "Paseo");
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    await waitFor(() => expect(asked("diary_write")).toHaveLength(1));
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    await userEvent.click(screen.getByRole("button", { name: t("diaryKeep") }));

    expect(asked("diary_write")).toHaveLength(1);
  });

  it("sticks a short note in today on Enter once the note is chosen", async () => {
    const onWritten = vi.fn();
    box(onWritten);

    await userEvent.click(screen.getByRole("button", { name: t("diaryStick") }));
    const field = screen.getByRole("textbox", { name: t("diaryStick") });
    expect(field.getAttribute("maxLength")).toBe(String(IDEA_AT_MOST));
    await userEvent.type(field, "Comprar pan{Enter}");

    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(asked("diary_stick")).toEqual([{ cmd: "diary_stick", args: { note: "Comprar pan" } }]);
    expect(asked("diary_write")).toEqual([]);
  });
});

describe("the diary's head", () => {
  it("says only you see it, turns the door when asked, and offers the month as a document", async () => {
    const onTurn = vi.fn();
    const onEdit = vi.fn();
    const { rerender } = render(
      <DiaryHead
        over="Noviembre 2026"
        big="Domingo 15"
        open={false}
        onTurn={onTurn}
        onEdit={onEdit}
      />,
    );

    expect(screen.getByRole("heading", { name: "Domingo 15" })).toBeTruthy();
    const door = screen.getByRole("button", { name: t("diaryHidden") });
    expect(door.getAttribute("aria-pressed")).toBe("false");
    await userEvent.click(door);
    await userEvent.click(screen.getByRole("button", { name: t("diaryEditMonth") }));
    expect(onTurn).toHaveBeenCalledOnce();
    expect(onEdit).toHaveBeenCalledOnce();

    rerender(<DiaryHead over="" big="Diario 2026" open onTurn={onTurn} />);
    expect(screen.getByRole("button", { name: t("diaryOpen") })).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("diaryEditMonth") })).toBeNull();
  });
});
