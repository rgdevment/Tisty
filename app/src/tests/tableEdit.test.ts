import { describe, expect, it, vi } from "vitest";
import { tableEdit, tableOrder, tableWritten } from "../tables";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve("| a |\n| --- |\n| 2 |\n");
  },
}));

describe("a change to a table", () => {
  it("goes to the core with the body, the table's place in bytes and the edit as the core reads it", async () => {
    const body = "| a |\n| --- |\n| 1 |\n";
    const done = await tableEdit(body, [0, 20], { kind: "cell", row: 0, column: 0, said: "2" });

    expect(done).toBe("| a |\n| --- |\n| 2 |\n");
    expect(ipc.calls).toEqual([
      {
        cmd: "table_edit",
        args: { body, at: [0, 20], edits: [{ kind: "cell", row: 0, column: 0, said: "2" }] },
      },
    ]);
  });

  it("asks the core for a column's order and for the words of the cells as written", async () => {
    ipc.calls.length = 0;
    await tableOrder("body", [3, 9], 2, true);
    await tableWritten("body", [3, 9]);
    expect(ipc.calls).toEqual([
      { cmd: "table_order", args: { body: "body", at: [3, 9], column: 2, down: true } },
      { cmd: "table_written", args: { body: "body", at: [3, 9] } },
    ]);
  });
});
