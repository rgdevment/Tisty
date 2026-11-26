import { describe, expect, it } from "vitest";
import { type Copy, copyKey, keyOf, timeline } from "../agentHistory";
import type { Earlier } from "../earlier";

const version = (print: string, wrote: string): Earlier => ({
  print,
  wrote,
  replaced: wrote,
  bytes: 10,
  here: true,
});

const copy = (n: number, when: string | undefined, more: Partial<Copy> = {}): Copy => ({
  at: "/home/me/.claude/CLAUDE.md",
  named: "Claude",
  n,
  bytes: 5,
  pinned: false,
  when,
  ...more,
});

const keys = (versions: Earlier[], copies: Copy[]) =>
  timeline(versions, copies).map((row) =>
    row.kind === "version" ? row.one.print : `#${row.one.n}`,
  );

describe("the history of an agent's file", () => {
  it("is empty when there is nothing to list", () => {
    expect(timeline([], [])).toEqual([]);
  });

  it("lists only the versions, in their order, when no copy was kept", () => {
    const all = [version("c", "2026-11-12T00:00:00Z"), version("a", "2026-11-10T00:00:00Z")];
    expect(keys(all, [])).toEqual(["c", "a"]);
  });

  it("lists only the copies, newest first, when there is no version", () => {
    const copies = [copy(1, "2026-11-10T00:00:00Z"), copy(2, "2026-11-12T00:00:00Z")];
    expect(keys([], copies)).toEqual(["#2", "#1"]);
  });

  it("slots each copy among the versions by its date and leaves the versions in their order", () => {
    const all = [
      version("c", "2026-11-14T00:00:00Z"),
      version("b", "2026-11-12T00:00:00Z"),
      version("a", "2026-11-10T00:00:00Z"),
    ];
    const copies = [
      copy(3, "2026-11-15T00:00:00Z"),
      copy(2, "2026-11-13T00:00:00Z"),
      copy(1, "2026-11-11T00:00:00Z"),
    ];
    expect(keys(all, copies)).toEqual(["#3", "c", "#2", "b", "#1", "a"]);
  });

  it("puts a copy older than every version after them", () => {
    const all = [version("a", "2026-11-10T00:00:00Z")];
    expect(keys(all, [copy(1, "2026-11-01T00:00:00Z")])).toEqual(["a", "#1"]);
  });

  it("keeps versions as they come even when their dates run the other way", () => {
    const all = [version("b", "2026-11-10T00:00:00Z"), version("a", "2026-11-12T00:00:00Z")];
    expect(keys(all, [copy(1, "2026-11-11T00:00:00Z")])).toEqual(["#1", "b", "a"]);
  });

  it("uses the date a version was replaced when it has no writing date", () => {
    const all: Earlier[] = [{ print: "x", replaced: "2026-11-12T00:00:00Z", bytes: 1, here: true }];
    expect(keys(all, [copy(1, "2026-11-13T00:00:00Z")])).toEqual(["#1", "x"]);
  });

  it("puts each file's original at the very end, however old the rest are", () => {
    const all = [version("a", "2026-11-10T00:00:00Z")];
    const copies = [
      copy(1, "2026-12-01T00:00:00Z", { pinned: true }),
      copy(4, "2026-11-20T00:00:00Z"),
      copy(1, "2026-12-02T00:00:00Z", { pinned: true, at: "/home/me/.codex/AGENTS.md" }),
    ];
    const rows = timeline(all, copies);
    expect(rows.map((row) => row.kind)).toEqual(["copy", "version", "copy", "copy"]);
    expect(rows.slice(-2).map((row) => (row.kind === "copy" ? row.one.at : ""))).toEqual([
      "/home/me/.claude/CLAUDE.md",
      "/home/me/.codex/AGENTS.md",
    ]);
  });

  it("keeps copies with no date in the order of their number, within their file", () => {
    const copies = [copy(1, undefined), copy(3, undefined), copy(2, undefined)];
    expect(keys([], copies)).toEqual(["#3", "#2", "#1"]);
  });

  it("places a copy with no date no later than the dated copy newer than it", () => {
    const all = [version("b", "2026-11-14T00:00:00Z"), version("a", "2026-11-10T00:00:00Z")];
    const copies = [copy(3, "2026-11-12T00:00:00Z"), copy(2, undefined)];
    expect(keys(all, copies)).toEqual(["b", "#3", "#2", "a"]);
  });

  it("interleaves the copies of two files by date", () => {
    const other = "/home/me/.codex/AGENTS.md";
    const copies = [
      copy(1, "2026-11-10T00:00:00Z"),
      copy(2, "2026-11-13T00:00:00Z"),
      copy(5, "2026-11-12T00:00:00Z", { at: other }),
      copy(4, "2026-11-11T00:00:00Z", { at: other }),
    ];
    expect(
      timeline([], copies).map((row) => (row.kind === "copy" ? `${row.one.at}#${row.one.n}` : "")),
    ).toEqual([
      "/home/me/.claude/CLAUDE.md#2",
      `${other}#5`,
      `${other}#4`,
      "/home/me/.claude/CLAUDE.md#1",
    ]);
  });

  it("names a copy by its file and number, and a version by its print", () => {
    const one = copy(2, undefined);
    expect(copyKey(one)).toBe("copy:2:/home/me/.claude/CLAUDE.md");
    expect(keyOf({ kind: "copy", one })).toBe(copyKey(one));
    expect(keyOf({ kind: "version", one: version("abc", "2026-11-10T00:00:00Z") })).toBe("abc");
  });
});
