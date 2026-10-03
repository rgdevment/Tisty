import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { List, Task } from "../core";
import Lists from "../ui/Lists";

const store = vi.hoisted(() => ({
  named: [] as { id: string; name: string }[],
  dropped: [] as string[],
  shelved: [] as string[],
  restored: [] as string[],
  asked: [] as string[],
  sure: true,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: (said: string) => {
    store.asked.push(said);
    return Promise.resolve(store.sure);
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "icons":
        return Promise.resolve([["home", "🏠"]]);
      case "list_rename":
        store.named.push({ id: String(args?.id), name: String(args?.name) });
        return Promise.resolve({ id: args?.id, name: args?.name, order: "a0" });
      case "list_drop":
        store.dropped.push(String(args?.id));
        return Promise.resolve(null);
      case "list_archive":
        store.shelved.push(String(args?.id));
        return Promise.resolve(null);
      case "list_unarchive":
        store.restored.push(String(args?.id));
        return Promise.resolve(null);
      default:
        return Promise.resolve(null);
    }
  },
}));

const lists: List[] = [
  { id: "01A", name: "Casa", order: "a0", icon: "home" },
  { id: "01B", name: "Trabajo", order: "a1" },
];

describe("tending a list", () => {
  const changed = vi.fn();

  beforeEach(() => {
    store.named = [];
    store.dropped = [];
    store.asked = [];
    store.sure = true;
    changed.mockClear();
  });

  const show = () =>
    render(
      <Lists
        lists={lists}
        counts={{ "01A": 2 }}
        soonest={{}}
        onOpen={vi.fn()}
        onChanged={changed}
        onError={vi.fn()}
      />,
    );

  const openFor = async (name: string) =>
    userEvent.click(await screen.findByLabelText(`Icon of ${name}`));

  it("gives a list a new name", async () => {
    show();
    await openFor("Casa");
    await userEvent.clear(screen.getByLabelText("Name of the list"));
    await userEvent.type(screen.getByLabelText("Name of the list"), "Hogar");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(store.named.length).toBe(1));
    expect(store.named[0]).toEqual({ id: "01A", name: "Hogar" });
    expect(changed).toHaveBeenCalled();
  });

  it("starts the renaming from the name it already has", async () => {
    show();
    await openFor("Trabajo");

    expect((screen.getByLabelText("Name of the list") as HTMLInputElement).value).toBe("Trabajo");
  });

  it("says nothing to the log when the name is left as it was", async () => {
    show();
    await openFor("Casa");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(store.named.length).toBe(0);
  });

  it("keeps the old name when the field is emptied", async () => {
    show();
    await openFor("Casa");
    await userEvent.clear(screen.getByLabelText("Name of the list"));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(store.named.length).toBe(0);
  });

  it("asks before deleting, and says where the open tasks go and what history keeps", async () => {
    show();
    await openFor("Casa");
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => expect(store.asked.length).toBe(1));
    expect(store.asked[0]).toContain("Casa");
    expect(store.asked[0]).toContain("goes to the inbox");
    expect(store.asked[0]).toContain("archived instead");
  });

  it("deletes the list once it is agreed", async () => {
    show();
    await openFor("Trabajo");
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => expect(store.dropped).toEqual(["01B"]));
    expect(changed).toHaveBeenCalled();
  });

  it("leaves the list alone when the asking is turned down", async () => {
    store.sure = false;
    show();
    await openFor("Trabajo");
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => expect(store.asked.length).toBe(1));
    expect(store.dropped.length).toBe(0);
    expect(changed).not.toHaveBeenCalled();
  });
});

describe("what a list card says about what is inside", () => {
  const task = (id: string, title: string, at?: string, list = "01A"): Task => ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: `a${id}`,
    list,
    ...(at ? { date: { at, tz: "UTC", floating: true, has_time: false } } : {}),
  });

  const withTasks = (tasks: Task[]) =>
    render(
      <Lists
        lists={lists}
        counts={{ "01A": tasks.filter((one) => one.list === "01A").length }}
        soonest={{ "01A": tasks.filter((one) => one.list === "01A") }}
        onOpen={vi.fn()}
        onChanged={vi.fn()}
        onError={vi.fn()}
      />,
    );

  it("names them in the order it was handed, which is the one the core settled", () => {
    withTasks([
      task("3", "la cercana", "2020-01-01"),
      task("2", "la lejana", "2030-01-01"),
      task("1", "sin fecha"),
    ]);

    const said = screen.getAllByRole("button").map((one) => one.textContent);
    const order = said.filter((one) => one?.includes("fecha") || one?.includes("la "));
    expect(order[0]).toContain("la cercana");
    expect(order[1]).toContain("la lejana");
    expect(order[2]).toContain("sin fecha");
  });

  it("says it is settled where nothing is open, rather than leaving the card bare", () => {
    withTasks([]);

    expect(screen.getAllByText("Nothing open").length).toBe(lists.length);
  });

  it("marks a task with no date so the column beside it is not left short", () => {
    withTasks([task("1", "algún día")]);

    expect(screen.getByText("—")).toBeTruthy();
  });
});

describe("putting a list away", () => {
  const changed = vi.fn();
  const every: List[] = [...lists, { id: "01C", name: "Moving", order: "a2", archived: true }];

  beforeEach(() => {
    store.shelved = [];
    store.restored = [];
    changed.mockClear();
  });

  const show = () =>
    render(
      <Lists
        lists={every}
        counts={{ "01A": 2 }}
        soonest={{}}
        onOpen={vi.fn()}
        onChanged={changed}
        onError={vi.fn()}
      />,
    );

  it("keeps an archived list apart from the ones in use", () => {
    show();

    const away = screen.getByRole("region", { name: "Archived" });
    expect(away.textContent).toContain("Moving");
    expect(screen.queryByLabelText("Icon of Moving")).toBeNull();
  });

  it("offers to archive only a list with nothing open", async () => {
    show();

    await userEvent.click(await screen.findByLabelText("Icon of Casa"));
    expect(screen.queryByRole("button", { name: "Archive" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

    await userEvent.click(await screen.findByLabelText("Icon of Trabajo"));
    await userEvent.click(screen.getByRole("button", { name: "Archive" }));

    await waitFor(() => expect(store.shelved).toEqual(["01B"]));
    expect(changed).toHaveBeenCalled();
  });

  it("brings an archived list back", async () => {
    show();

    await userEvent.click(screen.getByRole("button", { name: "Restore Moving" }));

    await waitFor(() => expect(store.restored).toEqual(["01C"]));
  });
});

describe("an archived list somebody filed into", () => {
  it("stands with the lists in use while it holds open work", () => {
    render(
      <Lists
        lists={[{ id: "01C", name: "Moving", order: "a2", archived: true }]}
        counts={{ "01C": 1 }}
        soonest={{}}
        onOpen={vi.fn()}
        onChanged={vi.fn()}
        onError={vi.fn()}
      />,
    );

    expect(screen.getByLabelText("Icon of Moving")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Archived" })).toBeNull();
  });
});
