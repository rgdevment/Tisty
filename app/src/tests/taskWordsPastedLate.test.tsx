import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import TaskWords from "../ui/TaskWords";

const BODY = "# Compras\n\nleche\n";
const DRAWN =
  '<h1 data-at="0-10"><span data-src="2-9">Compras</span></h1>' +
  '<p data-at="11-17"><span data-src="11-16">leche</span></p>';

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  translating: [] as ((said: string) => void)[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "task_doc_open") return Promise.resolve({ file: "", body: BODY, held: false });
    if (cmd === "task_step_dates") return Promise.resolve([]);
    if (cmd === "task_doc_make") return Promise.resolve("t_one-0001");
    if (cmd === "doc_drawn" || cmd === "doc_placed")
      return Promise.resolve(new TextEncoder().encode(DRAWN).buffer);
    if (cmd === "doc_from_html")
      return new Promise((done) => {
        ipc.translating.push(done);
      });
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const task = { id: "one", title: "Compras", status: "open" } as unknown as Task;

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).map((one) => one.args);

const loaded = async (container: HTMLElement) => {
  const frame = await waitFor(() => {
    const found = container.querySelector("iframe");
    expect(found?.srcdoc).toContain("<article");
    return found as HTMLIFrameElement;
  });
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  await waitFor(() =>
    expect(page.querySelector(".page")?.getAttribute("contenteditable")).toBe("true"),
  );
  return page;
};

const asked = (page: Document, inputType: string, more: Record<string, unknown>) => {
  const words = page.querySelector('.page [data-src="11-16"]')?.firstChild;
  if (words) page.getSelection()?.collapse(words, 5);
  const event = new InputEvent("beforeinput", { inputType, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  for (const [name, value] of Object.entries(more)) Object.defineProperty(event, name, { value });
  page.querySelector(".page")?.dispatchEvent(event);
};

beforeEach(() => {
  ipc.calls = [];
  ipc.translating = [];
});

describe("a new task's page pasted on while its first words make its file", () => {
  it("keeps what was pasted on that page, written into the file it took", async () => {
    const failed: unknown[] = [];
    const view = render(
      <TaskWords
        task={task}
        onSettle={() => Promise.resolve(null)}
        onError={(e) => failed.push(e)}
      />,
    );
    const page = await loaded(view.container);

    asked(page, "insertFromPaste", {
      dataTransfer: { getData: (kind: string) => (kind === "text/html" ? "<b>y</b>" : "y") },
    });
    await waitFor(() => expect(ipc.translating).toHaveLength(1));
    asked(page, "insertText", { data: "x" });
    await waitFor(() => expect(sent("task_doc_make")).toHaveLength(1), { timeout: 3000 });

    ipc.translating[0]("**y**");

    await waitFor(
      () =>
        expect(sent("doc_write")).toContainEqual(
          expect.objectContaining({ id: "t_one-0001", body: "# Compras\n\nleche**y**x\n" }),
        ),
      { timeout: 3000 },
    );
    expect(failed).toEqual([]);
  });
});
