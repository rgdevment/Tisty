import { describe, expect, it, vi } from "vitest";
import { tableEdit } from "../tables";

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
        args: { body, at: [0, 20], edit: { kind: "cell", row: 0, column: 0, said: "2" } },
      },
    ]);
  });
});
