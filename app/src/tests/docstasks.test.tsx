import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed, Task } from "../core";
import Docs from "../ui/Docs";

const held = vi.hoisted(() => ({
  sheet: null as null | {
    onTask?: (id: string) => void;
    cited?: (id: string) => { kind: string };
    cards?: string;
  },
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "doc_read") return Promise.resolve("# Auto\n\n![Neumáticos](tisty:task/01K)\n");
    if (cmd === "task_of")
      return args?.id === "01K"
        ? Promise.resolve({ id: "01K", title: "Cambiar los neumáticos", status: "done" })
        : Promise.reject({ code: "notATaskId" });
    if (cmd === "task_cards")
      return Promise.resolve([
        { id: "01K", title: "Cambiar los neumáticos", status: "done", steps: [] },
      ]);
    return Promise.resolve(null);
  }),
}));

vi.mock("../ui/Sheet", () => ({
  default: (props: NonNullable<typeof held.sheet>) => {
    held.sheet = props;
    return <div />;
  },
}));

const known: Filed[] = [
  {
    id: "01A",
    file: "auto-0001",
    title: "Auto",
    folder: null,
    archived: false,
    away: false,
  } as Filed,
];

describe("a task cited in a document", () => {
  it("is read for its card, opens from it, and a missing one opens nothing and says nothing", async () => {
    const onSeeTask = vi.fn<(task: Task) => void>();
    const onError = vi.fn();
    render(
      <Docs
        open="auto-0001"
        known={known}
        onKept={vi.fn()}
        onError={onError}
        onSeeTask={onSeeTask}
      />,
    );
    await waitFor(() => expect(held.sheet?.cited?.("01K").kind).toBe("closed"));

    held.sheet?.onTask?.("01K");
    await waitFor(() =>
      expect(onSeeTask).toHaveBeenCalledWith(expect.objectContaining({ id: "01K" })),
    );

    held.sheet?.onTask?.("01GONE");
    await new Promise((done) => setTimeout(done, 20));
    expect(onError).not.toHaveBeenCalled();
    expect(onSeeTask).toHaveBeenCalledTimes(1);
  });
});
