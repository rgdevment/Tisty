import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Machine } from "../core";
import { askAbout, done, knock, settled } from "../knocking";
import { fill, t } from "../locales";
import Knocking from "../ui/Knocking";

const calls: { cmd: string; args: unknown }[] = [];
let waiting: Machine[] = [];
let carried = "came";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: unknown) => {
    calls.push({ cmd, args });
    if (cmd === "waiting_machines") return Promise.resolve(waiting);
    if (cmd === "this_machine")
      return Promise.resolve({
        id: "dev_here",
        name: "ESCRITORIO",
        os: "Windows",
        code: "77120 03391 58804 21167",
      });
    if (cmd === "sync_now")
      return Promise.resolve({
        carried,
        undecided: [],
        unreadable: [],
        disowned: [],
        unconfirmed: [],
        astray: [],
        unprojected: false,
        joined: [],
      });
    return Promise.resolve(null);
  },
}));

const mac: Machine = {
  id: "dev_zwhwgd71",
  called: "pino 17",
  name: "MacBook Pro de Rodrigo",
  os: "macOS",
  when: 0,
  since: 1_791_250_000,
  mine: false,
  signs: "ab".repeat(32),
  code: "48213 90577 13046 62215",
  confirmed: null,
  confirmedWhen: 0,
  turnedAway: "unconfirmed",
};

const flush = () => act(() => new Promise((ready) => setTimeout(ready, 0)));

beforeEach(() => {
  calls.length = 0;
  waiting = [mac];
  carried = "came";
});

afterEach(() =>
  act(() => {
    done();
    settled();
  }),
);

describe("a computer waiting to be confirmed", () => {
  it("is named in the sidebar by what the computer calls itself", async () => {
    render(<Knocking />);
    await act(() => knock());

    expect(screen.getByText(fill("knockOne", "MacBook Pro de Rodrigo"))).toBeTruthy();
    expect(screen.queryByText("pino 17")).toBeNull();
  });

  it("asks to compare the code, and confirming brings in what it wrote", async () => {
    vi.useRealTimers();
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();

    expect(screen.getByText("48213 90577 13046 62215")).toBeTruthy();
    expect(screen.getByText(/77120 03391 58804 21167/)).toBeTruthy();

    waiting = [];
    fireEvent.click(screen.getByText(t("confirmYes")));
    await flush();
    await flush();

    expect(calls.find((one) => one.cmd === "confirm_machine_key")?.args).toEqual({
      id: "dev_zwhwgd71",
      key: "ab".repeat(32),
    });
    expect(calls.some((one) => one.cmd === "sync_now")).toBe(true);
    expect(screen.getByText(t("confirmBrought"))).toBeTruthy();
  });

  it("lets nothing in when the code does not match", async () => {
    vi.useRealTimers();
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();
    fireEvent.click(screen.getByText(t("confirmNo")));

    expect(screen.getByText(t("confirmMismatchWhat"))).toBeTruthy();
    expect(calls.some((one) => one.cmd === "confirm_machine_key")).toBe(false);
  });

  it("steps aside on «Not now» until another computer starts waiting", async () => {
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockLater")));
    expect(screen.queryByText(fill("knockOne", "MacBook Pro de Rodrigo"))).toBeNull();

    await act(() => knock());
    expect(screen.queryByText(fill("knockOne", "MacBook Pro de Rodrigo"))).toBeNull();

    waiting = [mac, { ...mac, id: "dev_other", name: "iMac" }];
    await act(() => knock());
    expect(screen.getByText(fill("knockMany", "2"))).toBeTruthy();
  });

  it("does not say everything arrived when another round held the folder", async () => {
    vi.useRealTimers();
    carried = "busy";
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();
    fireEvent.click(screen.getByText(t("confirmYes")));
    await flush();
    await flush();

    expect(screen.queryByText(t("confirmBrought"))).toBeNull();
    expect(screen.getByText(t("confirmLater"))).toBeTruthy();
  });

  it("says a machine that has not given its name is new, not what it is called", async () => {
    vi.useRealTimers();
    waiting = [{ ...mac, name: null, os: null }];
    render(<Knocking />);
    await act(() => knock());

    expect(screen.getByText(t("knockUnnamed"))).toBeTruthy();
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();
    expect(screen.getByText(fill("confirmUnnamed", "pino 17"))).toBeTruthy();
  });

  it("tells an assistant apart and where its code is", async () => {
    vi.useRealTimers();
    waiting = [{ ...mac, name: null, host: "ESCRITORIO-MARIO" }];
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();

    expect(screen.getByText(fill("confirmAgent", "ESCRITORIO-MARIO"))).toBeTruthy();
    expect(screen.getByText(t("confirmAgentStep"))).toBeTruthy();
  });

  it("moves on to the next machine after a code that does not match", async () => {
    vi.useRealTimers();
    waiting = [mac, { ...mac, id: "dev_imac", name: "iMac" }];
    render(<Knocking />);
    await act(() => knock());
    fireEvent.click(screen.getByText(t("knockConfirm")));
    await flush();
    fireEvent.click(screen.getByText(t("confirmNo")));
    fireEvent.click(screen.getByText(fill("confirmNext", "iMac")));

    expect(screen.getByText(fill("confirmTitle", "iMac"))).toBeTruthy();
  });

  it("looks for waiting machines as soon as it is shown", async () => {
    render(<Knocking />);
    await flush();

    expect(calls.some((one) => one.cmd === "waiting_machines")).toBe(true);
  });

  it("opens on the machine Maintenance asked about", async () => {
    vi.useRealTimers();
    render(<Knocking />);
    await act(() => askAbout({ ...mac, id: "dev_imac", name: "iMac" }));
    await flush();

    expect(screen.getByText(fill("confirmTitle", "iMac"))).toBeTruthy();
  });
});
