import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Notes from "../ui/Notes";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(cmd === "capture" ? { id: "01T", title: String(args?.text) } : null);
  },
}));

beforeEach(() => {
  ipc.calls = [];
});

const papers: Filed[] = [
  { id: "01A", file: "a3f1-0001", title: "Recetas", folder: null, archived: false, away: false },
  { id: "01B", file: "a3f1-0002", title: "Viejo", folder: null, archived: true, away: true },
  { id: "01C", file: "a3f1-0009", title: "Ideas", folder: null, archived: false, away: false },
];

const BODY =
  "---\nview: notes\n---\n\n# Ideas\n\n## 2026-11-10 18:40\n\nprimera\n\n## 2026-11-11 09:12\n\nsegunda\n";

describe("a brainstorm shown as notes", () => {
  it("puts the newest idea first", () => {
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={() => {}} />);

    const notes = within(screen.getByRole("list")).getAllByRole("listitem");
    expect(notes[0].textContent).toContain("segunda");
    expect(notes[1].textContent).toContain("primera");
  });

  it("writes a note back into its own section when it loses the focus", () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} />);

    const [newest] = screen.getAllByRole("textbox");
    fireEvent.change(newest, { target: { value: "segunda, mejorada" } });
    fireEvent.blur(newest);

    expect(onWrite).toHaveBeenCalledTimes(1);
    const written: string = onWrite.mock.calls[0][0];
    expect(written).toContain("## 2026-11-11 09:12\n\nsegunda, mejorada");
    expect(written).toContain("## 2026-11-10 18:40\n\nprimera");
  });

  it("does not write when nothing changed", () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} />);

    fireEvent.blur(screen.getAllByRole("textbox")[0]);

    expect(onWrite).not.toHaveBeenCalled();
  });

  it("removes one idea and keeps the other", async () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} />);

    await userEvent.click(screen.getAllByRole("button", { name: "Remove this idea" })[0]);

    const written: string = onWrite.mock.calls[0][0];
    expect(written).not.toContain("segunda");
    expect(written).toContain("primera");
  });

  it("only shows the ideas when the document is read-only", () => {
    render(<Notes title="Ideas" body={BODY} reading onWrite={() => {}} />);

    expect(screen.queryAllByRole("textbox")).toEqual([]);
    expect(screen.queryByRole("button", { name: "Remove this idea" })).toBeNull();
    expect(screen.queryByRole("button", { name: "New idea" })).toBeNull();
    expect(screen.getByText("primera")).toBeTruthy();
  });

  it("moves the focused idea one place with Alt and an arrow", async () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} />);

    fireEvent.keyDown(screen.getAllByRole("textbox")[0], { key: "ArrowRight", altKey: true });

    const written: string = onWrite.mock.calls[0][0];
    expect(written.indexOf("segunda")).toBeLessThan(written.indexOf("primera"));
  });

  it("paints each idea with the colour written in its heading, wherever it moves", () => {
    const painted = [
      "# Ideas",
      "## 2026-11-12 00:19 · blue",
      "una",
      "## 2026-11-12 00:19 · coral",
      "otra",
    ].join("\n\n");
    render(<Notes title="Ideas" body={painted} reading onWrite={() => {}} />);

    const hues = within(screen.getByRole("list"))
      .getAllByRole("listitem")
      .map((one) => one.dataset.hue);
    expect(hues).toEqual(["1", "8"]);
    expect(screen.queryByText(/blue|coral/)).toBeNull();
  });

  it("shows its Markdown only to read, so nothing but plain ideas gets in", async () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} />);

    await userEvent.click(screen.getByRole("button", { name: "Show Markdown" }));

    expect(screen.getByText(/## 2026-11-11 09:12/)).toBeTruthy();
    expect(screen.queryAllByRole("textbox")).toEqual([]);
    expect(screen.queryByRole("button", { name: "New idea" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Show notes" }));
    expect(screen.getAllByRole("textbox")).toHaveLength(2);
    expect(onWrite).not.toHaveBeenCalled();
  });

  it("holds an idea to 140 characters and counts down the last twenty", () => {
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={() => {}} />);
    const [newest] = screen.getAllByRole("textbox");

    expect(newest.getAttribute("maxlength")).toBe("140");
    expect(screen.queryByText("5")).toBeNull();
    fireEvent.change(newest, { target: { value: "a".repeat(135) } });
    expect(screen.getByText("5")).toBeTruthy();
  });

  it("turns an idea into a task, read as any capture, and takes it off the sheet", async () => {
    const onWrite = vi.fn();
    const onTask = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} onTask={onTask} />);

    await userEvent.click(screen.getAllByRole("button", { name: "Turn into a task" })[0]);

    await waitFor(() => expect(onWrite).toHaveBeenCalledTimes(1));
    expect(ipc.calls.find((one) => one.cmd === "capture")?.args.text).toBe("segunda");
    const written: string = onWrite.mock.calls[0][0];
    expect(written).not.toContain("segunda");
    expect(written).toContain("primera");
    expect(screen.getByRole("status").textContent).toContain("Now a task: «segunda»");
    await userEvent.click(screen.getByRole("button", { name: "See the task" }));
    expect(onTask).toHaveBeenCalledWith(expect.objectContaining({ title: "segunda" }));
  });

  it("files an idea in a document it picks, never itself nor one put away", async () => {
    const onWrite = vi.fn();
    const onDoc = vi.fn();
    render(
      <Notes
        title="Ideas"
        body={BODY}
        reading={false}
        onWrite={onWrite}
        known={papers}
        own="a3f1-0009"
        onDoc={onDoc}
      />,
    );

    await userEvent.click(screen.getAllByRole("button", { name: "File in a document" })[1]);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).queryByRole("button", { name: "Ideas" })).toBeNull();
    expect(within(dialog).queryByRole("button", { name: "Viejo" })).toBeNull();
    await userEvent.click(within(dialog).getByRole("button", { name: "Recetas" }));

    await waitFor(() => expect(onWrite).toHaveBeenCalledTimes(1));
    expect(ipc.calls.find((one) => one.cmd === "idea_to_doc")?.args).toEqual({
      file: "a3f1-0001",
      idea: "primera",
    });
    expect(onWrite.mock.calls[0][0]).not.toContain("primera");
    expect(screen.getByRole("status").textContent).toContain("Filed at the end of «Recetas»");
    await userEvent.click(screen.getByRole("button", { name: "Open it" }));
    expect(onDoc).toHaveBeenCalledWith("a3f1-0001");
  });

  it("keeps the idea on the sheet when it could not be filed", async () => {
    const onWrite = vi.fn();
    const onError = vi.fn();
    ipc.calls = [];
    render(
      <Notes
        title="Ideas"
        body={BODY}
        reading={false}
        onWrite={onWrite}
        known={[]}
        own="a3f1-0009"
        onError={onError}
      />,
    );

    await userEvent.click(screen.getAllByRole("button", { name: "File in a document" })[0]);

    expect(within(screen.getByRole("dialog")).getByText("No document by that name")).toBeTruthy();
    expect(onWrite).not.toHaveBeenCalled();
  });
});
