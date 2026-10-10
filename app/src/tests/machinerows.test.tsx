import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Machine } from "../core";
import { done, knocking, settled } from "../knocking";
import { fill, t } from "../locales";
import { hushed, hushedName, MachineList } from "../ui/Keys";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const one = (over: Partial<Machine>): Machine => ({
  id: "dev_mac",
  called: "pino 17",
  name: "MacBook Pro de Rodrigo",
  os: "macOS",
  when: 1_791_250_000,
  since: 1_791_200_000,
  mine: false,
  signs: "ab".repeat(32),
  code: "48213 90577 13046 62215",
  confirmed: null,
  confirmedWhen: 0,
  turnedAway: null,
  ...over,
});

const listed = (all: Machine[]) =>
  render(
    <MachineList all={all} busy={false} onKey={vi.fn()} onAstray={vi.fn()} onDrop={vi.fn()} />,
  );

afterEach(() => {
  done();
  settled();
});

describe("the machines in Maintenance", () => {
  it("are named by what the computer calls itself, with its system and its code", () => {
    listed([one({ confirmed: "ab".repeat(32), confirmedWhen: 1_791_200_000 })]);

    expect(screen.getByText("MacBook Pro de Rodrigo")).toBeTruthy();
    expect(screen.getByText(/macOS/)).toBeTruthy();
    expect(screen.getByText("48213 90577 13046 62215")).toBeTruthy();
    expect(screen.queryByText("pino 17")).toBeNull();
  });

  it("say when a key was taken on update rather than compared", () => {
    listed([one({ confirmed: "ab".repeat(32), carried: true })]);

    expect(screen.getByText(t("machineCarried"))).toBeTruthy();
  });

  it("say when a key was followed from the one confirmed before", () => {
    listed([one({ confirmed: "ab".repeat(32), rotated: true })]);

    expect(screen.getByText(t("machineRotated"))).toBeTruthy();
    expect(screen.queryByText(t("machineCarried"))).toBeNull();
  });

  it("show a waiting machine as writing since it began, never as dormant", () => {
    const waiting = one({ when: 0, turnedAway: "unconfirmed" });
    listed([waiting]);

    expect(screen.queryByText(t("machineNever"))).toBeNull();
    expect(screen.getByText(new RegExp(fill("machineSince", "").trim()))).toBeTruthy();
    expect(hushed(waiting)).toBe(false);
  });

  it("open the confirm dialog on the waiting machine that was clicked", () => {
    listed([one({ when: 0, turnedAway: "unconfirmed" })]);

    fireEvent.click(screen.getByRole("button", { name: t("machineKeyConfirm") }));

    expect(knocking().asking).toBe(true);
    expect(knocking().waiting[0]?.id).toBe("dev_mac");
  });

  it("name a quiet machine by its computer name", () => {
    expect(hushedName([one({ when: 1 })])).toBe("MacBook Pro de Rodrigo");
    expect(hushedName([one({ when: 1, name: null })])).toBe("pino 17");
  });

  it("fall back to the key for a machine that has not said a code", () => {
    listed([one({ code: null, confirmed: "ab".repeat(32) })]);

    expect(screen.getByText(/abababab … abababab/)).toBeTruthy();
  });
});
