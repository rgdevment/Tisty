import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import TaskWords from "../ui/TaskWords";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  doc: null as unknown,
  sheets: [] as {
    file?: string;
    value: string;
    reading?: boolean;
    onWrite: (text: string) => void;
    onSettle?: (before: string, now: string) => void;
  }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "task_doc_open") return Promise.resolve(ipc.doc);
    if (cmd === "task_doc_make") {
      ipc.doc = { file: "t_one-0001", body: "---\ntask: one\n---\n", held: false };
      return Promise.resolve("t_one-0001");
    }
    return Promise.resolve({ id: String(args?.id ?? ""), title: "" });
  },
}));

vi.mock("../ui/Sheet", () => ({
  default: (props: (typeof ipc.sheets)[number]) => {
    ipc.sheets.push(props);
    return <p>{`sheet:${props.value}`}</p>;
  },
}));

const task = (status: Task["status"] = "open") =>
  ({ id: "one", title: "Rina's booster", status }) as unknown as Task;

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  ipc.calls = [];
  ipc.doc = null;
  ipc.sheets = [];
});

describe("a task with no document yet", () => {
  it("shows what it held and offers to write it as a document", async () => {
    render(<TaskWords task={task()} legacy={<p>old words</p>} onSettle={() => {}} />);

    expect(await screen.findByText("old words")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: t("writeAsDoc") }));

    expect(await screen.findByText(/^sheet:/)).toBeTruthy();
    expect(sent("task_doc_make")).toEqual([{ cmd: "task_doc_make", args: { id: "one" } }]);
    expect(sent("task_doc_open")).toHaveLength(2);
  });

  it("offers nothing on a closed task, nor where nobody listens for changes", async () => {
    const { unmount } = render(
      <TaskWords task={task("done")} legacy={<p>old words</p>} onSettle={() => {}} />,
    );
    await screen.findByText("old words");
    expect(screen.queryByRole("button", { name: t("writeAsDoc") })).toBeNull();
    unmount();

    render(<TaskWords task={task()} legacy={<p>old words</p>} />);
    await screen.findByText("old words");
    expect(screen.queryByRole("button", { name: t("writeAsDoc") })).toBeNull();
  });

  it("takes an answer that is no document as none", async () => {
    ipc.doc = { id: "one", title: "Rina's booster" };
    render(<TaskWords task={task()} legacy={<p>old words</p>} />);

    expect(await screen.findByText("old words")).toBeTruthy();
    expect(ipc.sheets).toHaveLength(0);
  });
});

describe("a task with its document", () => {
  it("writes what changed once the typing settles, and hands each settled change on", async () => {
    ipc.doc = { file: "t_one-0001", body: "- [ ] uno\n", held: false };
    const settled: [string, string][] = [];
    render(
      <TaskWords
        task={task()}
        legacy={<p>old words</p>}
        onSettle={(before, now) => settled.push([before, now])}
      />,
    );

    await screen.findByText("sheet:- [ ] uno");
    expect(screen.queryByText("old words")).toBeNull();
    const sheet = ipc.sheets[ipc.sheets.length - 1];
    expect(sheet.file).toBe("t_one-0001");
    expect(sheet.reading).toBe(false);

    sheet.onWrite("- [x] uno\n");
    sheet.onSettle?.("- [ ] uno\n", "- [x] uno\n");
    expect(settled).toEqual([["- [ ] uno\n", "- [x] uno\n"]]);
    expect(sent("doc_write")).toHaveLength(0);

    await waitFor(
      () =>
        expect(sent("doc_write")).toEqual([
          { cmd: "doc_write", args: { id: "t_one-0001", body: "- [x] uno\n", anyway: undefined } },
        ]),
      { timeout: 2000 },
    );
  });

  it("is only read while it is held, and tells no changes", async () => {
    ipc.doc = { file: "t_one-0001", body: "words\n", held: true };
    render(<TaskWords task={task()} legacy={<p>old words</p>} onSettle={() => {}} />);

    await screen.findByText("sheet:words");
    const sheet = ipc.sheets[ipc.sheets.length - 1];
    expect(sheet.reading).toBe(true);
    expect(sheet.onSettle).toBeUndefined();
  });
});
