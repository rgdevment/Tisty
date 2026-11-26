import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { t } from "../locales";
import TaskWords from "../ui/TaskWords";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  doc: null as unknown,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "task_doc_open") return Promise.resolve(ipc.doc);
    if (cmd === "task_step_dates") return Promise.resolve([]);
    if (cmd === "doc_write" && !args.anyway) {
      return Promise.reject({ code: "documentMoved", name: "t_one-0001" });
    }
    if (cmd === "task_doc_merge") return Promise.resolve(args.prefer === "mine");
    return Promise.resolve({ id: String(args?.id ?? ""), title: "" });
  },
}));

vi.mock("../ui/Sheet", () => ({
  default: (props: { value: string; onWrite: (text: string) => void }) => {
    sheets.push(props);
    return <p>{`sheet:${props.value}`}</p>;
  },
}));

const sheets: { value: string; onWrite: (text: string) => void }[] = [];
const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  ipc.calls = [];
  sheets.length = 0;
});

it("a task's page saved with mine anyway goes through the merge preferring mine, never a plain write", async () => {
  ipc.doc = { file: "t_one-0001", body: "mine\n", held: false };
  render(<TaskWords task={{ id: "one", title: "Rina's booster", status: "open" } as Task} />);
  await screen.findByText("sheet:mine");
  sheets[sheets.length - 1].onWrite("mine, edited\n");
  // The plain merge refuses (the mock answers false), so the bar offers mine.
  fireEvent.click(await screen.findByText(t("clashSave"), {}, { timeout: 4000 }));

  await waitFor(() =>
    expect(sent("task_doc_merge").map((one) => one.args)).toContainEqual({
      id: "one",
      base: "mine\n",
      mine: "mine, edited\n",
      prefer: "mine",
    }),
  );
  expect(sent("doc_write").filter((one) => one.args.anyway)).toHaveLength(0);
});
