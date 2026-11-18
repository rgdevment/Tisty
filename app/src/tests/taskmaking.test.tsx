import { fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { fill, t } from "../locales";
import Origin from "../ui/Origin";
import Picking, { type Picked } from "../ui/Picking";
import { useTaskAsks } from "../ui/taskAsking";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  refused: null as { code: string; name?: string } | null,
  held: null as Promise<void> | null,
  emitted: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    switch (cmd) {
      case "snapshot":
        return Promise.resolve({ tasks: [], lists: [], tags: [], counts: {}, locale: "en" });
      case "read":
        return Promise.resolve({
          title: String(args?.text ?? ""),
          tags: [],
          spans: [],
          offers: [],
        });
      case "capture": {
        if (ipc.refused) return Promise.reject(ipc.refused);
        const task = { id: "01T", title: "Comprar las uvas", status: "open" };
        return (ipc.held ?? Promise.resolve()).then(() => task);
      }
      case "doc_heads":
        return Promise.resolve([{ id: "ano-nuevo", title: "Año nuevo", level: 2 }]);
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: (name: string) => {
    ipc.emitted.push(name);
    return Promise.resolve();
  },
  listen: () => Promise.resolve(() => {}),
}));

const captured = () => ipc.calls.filter((one) => one.cmd === "capture");

const opened = (asked: Partial<Picked> = {}) => {
  const put = vi.fn((_: string) => true);
  const onDone = vi.fn();
  const onError = vi.fn();
  const picked: Picked = {
    kind: "newTask",
    at: { x: 10, y: 10 },
    put,
    file: "a-0001",
    before: "# Viaje\n\n## Año nuevo\n\n",
    ...asked,
  };
  render(
    <Picking
      picked={{ ...picked, put: asked.put ?? put }}
      pages={[]}
      others={[]}
      onMade={() => {}}
      onLost={() => {}}
      onError={onError}
      onDone={onDone}
    />,
  );
  return { put, onDone, onError, field: screen.getByRole("textbox") };
};

describe("a new task written from the / menu", () => {
  beforeEach(() => {
    ipc.calls = [];
    ipc.refused = null;
    ipc.held = null;
    ipc.emitted = [];
  });

  it("is read as the quick capture reads it and leaves its card where it was asked for", async () => {
    const { put, onDone, field } = opened();

    await userEvent.type(field, "Comprar las uvas el lunes #compras{Enter}");

    await waitFor(() => expect(put).toHaveBeenCalledWith("![Comprar las uvas](tisty:task/01T)"));
    expect(captured()).toHaveLength(1);
    expect(captured()[0]?.args).toMatchObject({
      text: "Comprar las uvas el lunes #compras",
      born: { doc: "a-0001", before: "# Viaje\n\n## Año nuevo\n\n" },
    });
    expect(onDone).toHaveBeenCalled();
    expect(ipc.emitted).toContain("captured");
  });

  it("is offered on a task's page only where that task can hold parts", () => {
    const kinds = (whole?: string) =>
      renderHook(() => useTaskAsks([], undefined, whole)).result.current.asking?.kinds;
    expect(kinds("01W")).toContain("newTask");
    expect(kinds()).not.toContain("newTask");
    expect(kinds()).toContain("task");
  });

  it("on a task's page is made a part of that task", async () => {
    const { put, field } = opened({ whole: "01W", file: undefined, before: undefined });

    await userEvent.type(field, "Embalar los libros{Enter}");

    await waitFor(() => expect(put).toHaveBeenCalled());
    expect(captured()[0]?.args.born).toEqual({ whole: "01W" });
  });

  it("makes nothing from an empty line and stays open to be written", async () => {
    const { put, onDone, field } = opened();

    await userEvent.type(field, "   {Enter}");

    expect(await screen.findByRole("alert")).toHaveProperty("textContent", t("untitled"));
    expect(captured()).toEqual([]);
    expect(put).not.toHaveBeenCalled();
    expect(onDone).not.toHaveBeenCalled();
  });

  it("says why when the task cannot be made, and leaves no card", async () => {
    ipc.refused = { code: "noSuchList", name: "Casa" };
    const { put, onDone, field } = opened();

    await userEvent.type(field, "Comprar las uvas @Casa{Enter}");

    expect(await screen.findByRole("alert")).toHaveProperty(
      "textContent",
      fill("noSuchList", "Casa"),
    );
    expect(put).not.toHaveBeenCalled();
    expect(onDone).not.toHaveBeenCalled();
  });

  it("makes one task however often Enter is pressed while it is being made", async () => {
    let release = () => {};
    ipc.held = new Promise((done) => {
      release = done;
    });
    const { put, field } = opened();

    await userEvent.type(field, "Comprar las uvas{Enter}{Enter}");
    fireEvent.keyDown(field, { key: "Escape" });
    release();

    await waitFor(() => expect(put).toHaveBeenCalledTimes(1));
    expect(captured()).toHaveLength(1);
  });

  it("is let go with Escape, and Escape inside its own menu closes only that menu", async () => {
    const { put, onDone, field } = opened();

    await userEvent.type(field, "Comprar /");
    expect(await screen.findByRole("listbox", { name: t("insertLabel") })).toBeTruthy();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(onDone).not.toHaveBeenCalled();

    await userEvent.keyboard("{Escape}");
    expect(onDone).toHaveBeenCalled();
    expect(captured()).toEqual([]);
    expect(put).not.toHaveBeenCalled();
  });

  it("says the task is there when the place for its card is gone", async () => {
    const { onError, field } = opened({ put: () => false });

    await userEvent.type(field, "Comprar las uvas{Enter}");

    await waitFor(() => expect(onError).toHaveBeenCalledWith(t("taskMadeCardLost")));
  });
});

const PAPERS = [
  { id: "01D", file: "a-0001", title: "Viaje", folder: null, archived: false, away: false },
] as Filed[];

describe("where a task was born, beside its page", () => {
  beforeEach(() => {
    ipc.calls = [];
  });

  it("names the document and the section, and opens the document", async () => {
    const onDoc = vi.fn();
    render(<Origin source="a-0001#ano-nuevo" papers={PAPERS} onDoc={onDoc} />);

    expect(screen.getByText(t("bornIn"))).toBeTruthy();
    expect(await screen.findByText(fill("citedUnder", "Año nuevo"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button"));
    expect(onDoc).toHaveBeenCalledWith("a-0001");
  });

  it("says nothing of a source that names no document here", () => {
    for (const source of [undefined, "sereno#1234", "zz-0009#top", "a-0001"]) {
      const { container, unmount } = render(<Origin source={source} papers={PAPERS} />);
      expect(container.textContent).toBe("");
      unmount();
    }
  });
});
