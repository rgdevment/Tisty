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
  it("reads as its template until written, and its first words make the document", async () => {
    ipc.doc = { file: "", body: "Template.\n", held: false };
    const settled: [string, string][] = [];
    render(
      <TaskWords
        task={task()}
        onSettle={(before, now) => Promise.resolve(settled.push([before, now]) > 0 ? null : "x")}
      />,
    );

    await screen.findByText("sheet:Template.");
    expect(sent("task_doc_open")[0].args).toEqual({ id: "one" });
    expect(sent("task_doc_make")).toHaveLength(0);

    const sheet = ipc.sheets[ipc.sheets.length - 1];
    sheet.onWrite("Mine.\n");
    sheet.onSettle?.("Template.\n", "Mine.\n");
    await waitFor(() => expect(settled).toHaveLength(1));
    expect(sent("task_doc_make")).toEqual([
      {
        cmd: "task_doc_make",
        args: { id: "one", body: "Mine.\n" },
      },
    ]);
  });

  it("is not made again once it has its document", async () => {
    ipc.doc = { file: "t_one-0001", body: "words\n", held: false };
    render(<TaskWords task={task()} />);

    await screen.findByText("sheet:words");
    expect(sent("task_doc_make")).toHaveLength(0);
  });

  it("takes an answer that is no document as none, and makes nothing", async () => {
    ipc.doc = { id: "one", title: "Rina's booster" };
    render(<TaskWords task={task()} />);

    await waitFor(() => expect(sent("task_doc_open")).toHaveLength(1));
    await settle();
    expect(sent("task_doc_make")).toHaveLength(0);
    expect(ipc.sheets).toHaveLength(0);
  });
});

describe("a closed task", () => {
  it("gets no document it never had", async () => {
    render(<TaskWords task={task("done")} />);
    await waitFor(() => expect(sent("task_doc_open")).toHaveLength(1));
    await settle();

    expect(sent("task_doc_make")).toHaveLength(0);
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
        onSettle={(before, now) =>
          Promise.resolve(settled.push([before, now]) > 0 ? null : "failed")
        }
      />,
    );

    await screen.findByText("sheet:- [ ] uno");
    const sheet = ipc.sheets[ipc.sheets.length - 1];
    expect(sheet.file).toBe("t_one-0001");
    expect(sheet.reading).toBe(false);
    expect(sheet.inline).toBe(true);

    sheet.onWrite("- [x] uno\n");
    sheet.onSettle?.("- [ ] uno\n", "- [x] uno\n");
    await waitFor(() => expect(settled).toEqual([["- [ ] uno\n", "- [x] uno\n"]]));
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
    render(<TaskWords task={task()} onSettle={() => Promise.resolve(null)} />);

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

describe("a settle that failed", () => {
  it("hands its text before to the next one, so no line goes unlearnt", async () => {
    ipc.doc = { file: "t_one-0001", body: "a\n", held: false };
    const asked: [string, string][] = [];
    let answer: string | null = "failed";
    render(
      <TaskWords
        task={task()}
        onSettle={(before, now) => {
          asked.push([before, now]);
          return Promise.resolve(answer);
        }}
      />,
    );
    await screen.findByText("sheet:a");
    const sheet = ipc.sheets[ipc.sheets.length - 1];

    sheet.onSettle?.("a\n", "b\n");
    await waitFor(() => expect(asked).toHaveLength(1));
    answer = null;
    sheet.onSettle?.("b\n", "c\n");
    await waitFor(() => expect(asked).toHaveLength(2));
    sheet.onSettle?.("c\n", "d\n");
    await waitFor(() => expect(asked).toHaveLength(3));

    expect(asked).toEqual([
      ["a\n", "b\n"],
      ["a\n", "c\n"],
      ["c\n", "d\n"],
    ]);
  });

  it("lets the next ones wait for it, so none weighs the same old text twice", async () => {
    ipc.doc = { file: "t_one-0001", body: "a\n", held: false };
    const asked: [string, string][] = [];
    const answers: (string | null)[] = ["failed", null, null];
    render(
      <TaskWords
        task={task()}
        onSettle={(before, now) => {
          asked.push([before, now]);
          return Promise.resolve(answers.shift() ?? null);
        }}
      />,
    );
    await screen.findByText("sheet:a");
    const sheet = ipc.sheets[ipc.sheets.length - 1];

    sheet.onSettle?.("a\n", "b\n");
    sheet.onSettle?.("b\n", "c\n");
    sheet.onSettle?.("c\n", "d\n");
    await waitFor(() => expect(asked).toHaveLength(3));

    expect(asked).toEqual([
      ["a\n", "b\n"],
      ["a\n", "c\n"],
      ["c\n", "d\n"],
    ]);
  });
});
