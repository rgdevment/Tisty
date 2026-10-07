import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Settled } from "../core";
import { fill, t } from "../locales";
import { syncSaid } from "../syncSaid";
import type { Which } from "../ui/Card";
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

const run = <T,>(_card: Which, work: Promise<T>, then: (answer: T) => void) => {
  void work.then(then);
};

const settled = (over: Partial<Settled>): Settled => ({
  carried: "same",
  came: 0,
  went: 0,
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
    render(<Here busy={null} run={run} tell={() => {}} />);
    await flush();

    expect(screen.getByText(fill("thisMachineIs", "ESCRITORIO"))).toBeTruthy();
    expect(screen.getByText("77120 03391 58804 21167")).toBeTruthy();
  });

  it("shows the code large, to be read from the other computer", async () => {
    render(<Here busy={null} run={run} tell={() => {}} />);
    await flush();

    fireEvent.click(screen.getByText(t("showBig")));

    expect(screen.getByRole("dialog", { name: fill("showBigTitle", "ESCRITORIO") })).toBeTruthy();
    expect(screen.getAllByText("77120 03391 58804 21167")).toHaveLength(2);
    fireEvent.click(screen.getByText(t("close")));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("renames this computer and says so", async () => {
    const told: string[] = [];
    render(<Here busy={null} run={run} tell={(word) => word && told.push(word.text)} />);
    await flush();

    fireEvent.click(screen.getByText(t("renameMachine")));
    fireEvent.change(screen.getByLabelText(t("renameMachine")), { target: { value: "Roble 42" } });
    fireEvent.click(screen.getByText(t("saveIt")));
    await flush();

    expect(calls).toContainEqual({ cmd: "rename_machine", args: { name: "Roble 42" } });
    expect(screen.getByText(fill("thisMachineIs", "Roble 42"))).toBeTruthy();
    expect(told).toEqual([fill("renamed", "Roble 42")]);
  });

  it("leaves the name alone when renaming is cancelled", async () => {
    render(<Here busy={null} run={run} tell={() => {}} />);
    await flush();

    fireEvent.click(screen.getByText(t("renameMachine")));
    fireEvent.click(screen.getByText(t("cancel")));

    expect(calls.some((one) => one.cmd === "rename_machine")).toBe(false);
    expect(screen.getByText(fill("thisMachineIs", "ESCRITORIO"))).toBeTruthy();
  });
});

describe("what a sync says it did", () => {
  const at = new Date(2026, 9, 6, 14, 32);
  const when = (said: string) => said.slice(said.indexOf(": ") + 2);

  it("counts what came, what went and what waits", () => {
    const said = syncSaid(
      settled({ came: 12, went: 1, unconfirmed: ["dev_w"], waiting: ["a", "b"] }),
      at,
    );

    expect(when(said)).toBe(
      [
        fill("syncResultCame", "12"),
        t("syncResultWentOne"),
        t("syncResultMachineWaits"),
        fill("syncResultDocsWait", "2"),
      ].join(", "),
    );
    expect(said.startsWith(fill("syncResultAt", ""))).toBe(true);
  });

  it("says nothing new when nothing moved", () => {
    expect(when(syncSaid(settled({}), at))).toBe(t("syncSame"));
  });

  it("says it was busy when another round was already running", () => {
    expect(syncSaid(settled({ carried: "busy" }), at)).toBe(t("syncBusy"));
  });
});

describe("what a sync says when it counted nothing", () => {
  it("still says something moved when the round says it did", () => {
    const said = syncSaid(settled({ carried: "came" }), new Date(2026, 9, 6, 14, 32));
    expect(said.endsWith(t("syncCame"))).toBe(true);
  });
});
