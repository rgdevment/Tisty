import { describe, expect, it } from "vitest";
import type { Facts } from "../core";
import { written } from "../report";

const seen: Facts = {
  version: "0.1.0",
  dev: true,
  sandbox: null,
  locale: "en",
  zone: "America/Santiago",
  os: "Windows 11 Pro 24H2 (10.0.26200.1234)",
  arch: "x86_64",
  webview: "132.0.2957.140",
  store: "C:\\Users\\···\\AppData\\Roaming\\tisty",
  devices: 2,
  events: 1481,
  open: 7,
  archived: 23,
  lists: 3,
  tags: 2,
  listNames: [],
  tagNames: [],
  cache: "agrees",
  attachments: 12,
  attachmentBytes: 1_700_000,
  loose: 3,
  looseBytes: 311_000,
  weight: 2_100_000,
  syncs: true,
  shared: true,
  backedUpAt: null,
  quiet: ["chime"],
  attachUpTo: 5 * 1024 * 1024,
  inPath: true,
  shortcut: "Ctrl+Shift+Space",
};

const at = new Date("2026-08-11T17:04:00");

describe("the report a bug gets attached to", () => {
  it("carries what somebody would have to ask for otherwise", () => {
    const paper = written(seen, at);

    expect(paper).toContain("0.1.0");
    expect(paper).toContain("Windows 11 Pro 24H2 (10.0.26200.1234)");
    expect(paper).toContain("x86_64");
    expect(paper).toContain("132.0.2957.140");
    expect(paper).toContain("America/Santiago");
    expect(paper).toContain("1481");
  });

  /// The counts stand whether or not the names do: a fault in the filters can
  /// be reproduced from «three lists», never from three words.
  it("puts placeholders where the names would be", () => {
    const paper = written(seen, at);

    expect(paper).toContain("lista#1, lista#2, lista#3");
    expect(paper).toContain("tag#1, tag#2");
  });

  it("uses the real names once they are asked for", () => {
    const paper = written(
      { ...seen, listNames: ["dentist", "house", "work"], tagNames: ["health", "home"] },
      at,
    );

    expect(paper).toContain("dentist, house, work");
    expect(paper).toContain("health, home");
    expect(paper).not.toContain("lista#1");
  });

  /// The store keeps the muted ones. A report that listed those would read as
  /// the list of what works.
  it("says which notices are on, not which are off", () => {
    const paper = written(seen, at);

    expect(paper).toMatch(/notices\s+A notification from the system/);
    expect(paper).not.toContain("A short tone");
  });

  it("says so when nothing would speak at all", () => {
    const paper = written({ ...seen, quiet: ["screen", "chime"] }, at);

    expect(paper).toMatch(/notices\s+none/);
  });

  /// Every line is «label  value», so a reader — and a person pasting it into
  /// an issue — can find a field without knowing the order.
  it("lines the values up under one another", () => {
    const paper = written(seen, at);
    const block = paper.slice(paper.indexOf("[system]"), paper.indexOf("[store]"));
    const columns = block
      .split("\n")
      .filter((line) => /^\w+ {2,}\S/.test(line))
      .map((line) => line.length - line.replace(/^\S+ +/, "").length);

    expect(new Set(columns).size).toBe(1);
  });

  it("says a sandbox is a sandbox, so nobody debugs the wrong store", () => {
    const paper = written({ ...seen, sandbox: "pruebas" }, at);

    expect(paper).toContain("pruebas");
  });

  it("leaves the sandbox line out where there is none", () => {
    expect(written(seen, at)).not.toContain("sandbox");
  });

  /// A shared folder IS the backup, so «none yet» there would read as a warning
  /// about something that is not missing.
  it("does not ask for a backup that the shared folder already is", () => {
    const paper = written(seen, at);

    expect(paper).toContain("the shared folder is the backup");
    expect(paper).not.toContain("none yet");
  });

  it("says when the last copy was made where copies are this machine's job", () => {
    const paper = written(
      { ...seen, shared: false, backedUpAt: "2026-08-04T10:00:00Z" },
      at,
    );

    expect(paper).toMatch(/backup\s+available/);
    expect(paper).not.toContain("none yet");
  });

  it("says «none yet» where no copy has ever been made", () => {
    const paper = written({ ...seen, shared: false, backedUpAt: null }, at);

    expect(paper).toContain("none yet");
  });

  /// The line that matters most, and the cheapest to lose in a refactor.
  it("says on its face that nothing has been sent", () => {
    expect(written(seen, at)).toContain("none of this leaves your machine");
  });
});
