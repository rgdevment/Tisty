import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Papers } from "../core";
import { t } from "../locales";
import { IDEA_AT_MOST } from "../notes";
import DiaryBar from "../ui/DiaryBar";
import Tree from "../ui/Tree";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  open: false,
}));

const TODAY = { book: "a3f1-0010", page: "a3f1-0011" };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "diary_of") return Promise.resolve({ book: TODAY.book, open: ipc.open });
    if (cmd === "diary_write" || cmd === "diary_stick") return Promise.resolve(TODAY);
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.calls = [];
  ipc.open = false;
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

  it("shows the books where they sit when nothing gathers them", () => {
    render(<Tree papers={papers} unfolded onOpen={vi.fn()} onFile={vi.fn()} />);

    expect(screen.queryByRole("button", { name: t("diary") })).toBeNull();
    expect(screen.getByRole("button", { name: "Diario 2026" })).toBeTruthy();
  });
});

describe("the diary's bar", () => {
  it("writes what is said into today and hands back where it landed", async () => {
    const onWritten = vi.fn();
    const onBefore = vi.fn(() => Promise.resolve());
    render(<DiaryBar onBefore={onBefore} onWritten={onWritten} onError={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: t("diaryWrite") }));
    await userEvent.type(screen.getByRole("textbox", { name: t("diaryWrite") }), "Paseo con Rina");
    await userEvent.keyboard("{Control>}{Enter}{/Control}");

    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(onBefore).toHaveBeenCalledOnce();
    expect(asked("diary_write")).toEqual([
      { cmd: "diary_write", args: { text: "Paseo con Rina" } },
    ]);
    expect(screen.queryByRole("textbox")).toBeNull();
  });

  it("sticks a short note in today on Enter", async () => {
    const onWritten = vi.fn();
    render(<DiaryBar onBefore={() => Promise.resolve()} onWritten={onWritten} onError={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: t("diaryStick") }));
    const field = screen.getByRole("textbox", { name: t("diaryStick") });
    expect(field.getAttribute("maxLength")).toBe(String(IDEA_AT_MOST));
    await userEvent.type(field, "Comprar pan{Enter}");

    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(asked("diary_stick")).toEqual([{ cmd: "diary_stick", args: { note: "Comprar pan" } }]);
  });

  it("says the diary is hidden from agents and opens it only when asked", async () => {
    render(<DiaryBar onBefore={() => Promise.resolve()} onWritten={vi.fn()} onError={vi.fn()} />);

    const door = await screen.findByRole("button", { name: t("diaryHidden") });
    expect(door.getAttribute("aria-pressed")).toBe("false");
    expect(asked("diary_agents")).toEqual([]);
    await userEvent.click(door);

    expect(asked("diary_agents")).toEqual([{ cmd: "diary_agents", args: { open: true } }]);
    expect(await screen.findByRole("button", { name: t("diaryOpen") })).toBeTruthy();
  });
});
