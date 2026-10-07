import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Settled } from "../core";
import { fill, t } from "../locales";
import { syncSaid } from "../syncSaid";
import Here from "../ui/Here";

const calls: { cmd: string; args: unknown }[] = [];
const machine = (name: string | null) => ({
  id: "dev_here",
  name,
  os: "Windows",
  code: "77120 03391 58804 21167",
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: { name?: string }) => {
    calls.push({ cmd, args });
    if (cmd === "this_machine") return Promise.resolve(machine("ESCRITORIO"));
    if (cmd === "rename_machine") return Promise.resolve(machine(args.name || "ESCRITORIO"));
    return Promise.resolve(null);
  },
}));

const flush = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

const settled = (over: Partial<Settled>): Settled => ({
  carried: "same",
  undecided: [],
  unreadable: [],
  disowned: [],
  unconfirmed: [],
  waiting: [],
  astray: [],
  unprojected: false,
  joined: [],
  ...over,
});

beforeEach(() => {
  calls.length = 0;
});

describe("this computer in Syncing", () => {
  it("says its name and its code", async () => {
    render(<Here held={false} />);
    await flush();

    expect(screen.getByText(fill("thisMachineIs", "ESCRITORIO"))).toBeTruthy();
    expect(screen.getByText("77120 03391 58804 21167")).toBeTruthy();
  });

  it("shows the code large, to be read from the other computer", async () => {
    render(<Here held={false} />);
    await flush();

    fireEvent.click(screen.getByText(t("showBig")));

    expect(screen.getByRole("dialog", { name: fill("showBigTitle", "ESCRITORIO") })).toBeTruthy();
    expect(screen.getAllByText("77120 03391 58804 21167")).toHaveLength(2);
    fireEvent.click(screen.getByText(t("close")));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("renames this computer and says so", async () => {
    render(<Here held={false} />);
    await flush();

    fireEvent.click(screen.getByText(t("renameMachine")));
    fireEvent.change(screen.getByLabelText(t("renameMachine")), { target: { value: "Roble 42" } });
    fireEvent.click(screen.getByText(t("saveIt")));
    await flush();

    expect(calls).toContainEqual({ cmd: "rename_machine", args: { name: "Roble 42" } });
    expect(screen.getByText(fill("thisMachineIs", "Roble 42"))).toBeTruthy();
    expect(screen.getByText(fill("renamed", "Roble 42"))).toBeTruthy();
  });

  it("leaves the name alone when renaming is cancelled", async () => {
    render(<Here held={false} />);
    await flush();

    fireEvent.click(screen.getByText(t("renameMachine")));
    fireEvent.click(screen.getByText(t("cancel")));

    expect(calls.some((one) => one.cmd === "rename_machine")).toBe(false);
    expect(screen.getByText(fill("thisMachineIs", "ESCRITORIO"))).toBeTruthy();
  });
});

describe("what a sync says it did", () => {
  const at = new Date(2026, 9, 6, 14, 32);

  it("says when it ran and what moved", () => {
    const said = syncSaid(settled({ carried: "both" }), at);

    expect(said.startsWith(fill("syncResultAt", ""))).toBe(true);
    expect(said.endsWith(t("syncBoth"))).toBe(true);
  });

  it("says nothing new when nothing moved", () => {
    expect(syncSaid(settled({}), at).endsWith(t("syncSame"))).toBe(true);
  });

  it("says it was busy when another round was already running", () => {
    expect(syncSaid(settled({ carried: "busy" }), at)).toBe(t("syncBusy"));
  });
});
