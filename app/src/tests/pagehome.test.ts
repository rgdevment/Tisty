import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { asksSure } from "../sure";
import { pageHome } from "../usePapers";

const ipc = vi.hoisted(() => ({
  calls: [] as Record<string, unknown>[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd !== "task_page_import") return Promise.resolve(null);
    ipc.calls.push(args ?? {});
    return args?.anyway ? Promise.resolve("t1") : Promise.reject({ code: "taskPageBehind" });
  },
}));

describe("an old copy of a task's page", () => {
  beforeEach(() => {
    ipc.calls = [];
  });

  it("comes in over what the task holds now only once the person says so", async () => {
    const said: string[] = [];
    asksSure((one) => {
      said.push(one);
      return Promise.resolve(true);
    });

    expect(await pageHome("/old.md")).toBe("t1");
    expect(said).toEqual([t("taskPageBehindSure")]);
    expect(ipc.calls.map((one) => one.anyway)).toEqual([undefined, true]);
  });

  it("stays out when the person keeps what the task holds", async () => {
    asksSure(() => Promise.resolve(false));

    expect(await pageHome("/old.md")).toBeUndefined();
    expect(ipc.calls).toHaveLength(1);
  });
});
