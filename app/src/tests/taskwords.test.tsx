import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import TaskWords from "../ui/TaskWords";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  doc: null as unknown,
  moved: false,
  sheets: [] as {
    file?: string;
    value: string;
    reading?: boolean;
    inline?: boolean;
    onWrite: (text: string) => void;
    onSettle?: (before: string, now: string) => void;
  }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "task_doc_open") return Promise.resolve(ipc.doc);
    if (cmd === "doc_write" && ipc.moved) return Promise.reject({ code: "documentMoved" });
    if (cmd === "task_doc_make") {
      ipc.doc = { file: "t_one-0001", body: "Words it held.\n", held: false };
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
const settle = () => new Promise((ready) => setTimeout(ready, 0));

beforeEach(() => {
  ipc.calls = [];
  ipc.doc = null;
  ipc.moved = false;
  ipc.sheets = [];
});

describe("an open task", () => {
  it("is a document from the first time it is opened, made from what it held", async () => {
    render(<TaskWords task={task()} onSettle={() => {}} />);

    expect(await screen.findByText("sheet:Words it held.")).toBeTruthy();
    expect(sent("task_doc_make")).toEqual([{ cmd: "task_doc_make", args: { id: "one" } }]);
    expect(sent("task_doc_open")).toHaveLength(2);
  });

  it("is not made again once it has its document", async () => {
    ipc.doc = { file: "t_one-0001", body: "words\n", held: false };
    render(<TaskWords task={task()} />);

    await screen.findByText("sheet:words");
    expect(sent("task_doc_make")).toHaveLength(0);
  });

  it("takes an answer that is no document as none, and asks for one", async () => {
    ipc.doc = { id: "one", title: "Rina's booster" };
    render(<TaskWords task={task()} />);

    await waitFor(() => expect(sent("task_doc_make")).toHaveLength(1));
  });
});

describe("a closed task", () => {
  it("gets no document it never had", async () => {
    render(<TaskWords task={task("done")} />);
    await settle();
    await settle();

    expect(sent("task_doc_make")).toHaveLength(0);
    expect(ipc.sheets).toHaveLength(0);
  });
});

describe("a task with its document", () => {
  it("writes what changed once the typing settles, and hands each settled change on", async () => {
    ipc.doc = { file: "t_one-0001", body: "- [ ] uno\n", held: false };
    const settled: [string, string][] = [];
    render(<TaskWords task={task()} onSettle={(before, now) => settled.push([before, now])} />);

    await screen.findByText("sheet:- [ ] uno");
    const sheet = ipc.sheets[ipc.sheets.length - 1];
    expect(sheet.file).toBe("t_one-0001");
    expect(sheet.reading).toBe(false);
    expect(sheet.inline).toBe(true);

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
    render(<TaskWords task={task()} onSettle={() => {}} />);

    await screen.findByText("sheet:words");
    const sheet = ipc.sheets[ipc.sheets.length - 1];
    expect(sheet.reading).toBe(true);
    expect(sheet.onSettle).toBeUndefined();
  });
});

describe("a task document changed on disk meanwhile", () => {
  it("reads it again and says so, instead of failing every save after", async () => {
    ipc.doc = { file: "t_one-0001", body: "mine\n", held: false };
    const failed = vi.fn();
    render(<TaskWords task={task()} onError={failed} />);
    await screen.findByText("sheet:mine");

    ipc.moved = true;
    ipc.doc = { file: "t_one-0001", body: "theirs\n", held: false };
    ipc.sheets[ipc.sheets.length - 1].onWrite("mine, edited\n");

    expect(await screen.findByText("sheet:theirs", {}, { timeout: 2000 })).toBeTruthy();
    expect(failed).toHaveBeenCalledWith({ code: "documentMoved" });
  });
});
