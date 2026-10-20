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
  unread: false,
  weight: 2_100_000,
  syncs: true,
  shared: true,
  backedUpAt: null,
  quiet: ["chime"],
  shortcut: "Ctrl+Shift+Space",
  installed: "download",
  assistant: false,
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

  it("puts placeholders where the names would be", () => {
    const paper = written(seen, at);

    expect(paper).toContain("list#1, list#2, list#3");
    expect(paper).toContain("tag#1, tag#2");
  });

  it("uses the real names once they are asked for", () => {
    const paper = written(
      { ...seen, listNames: ["dentist", "house", "work"], tagNames: ["health", "home"] },
      at,
    );

    expect(paper).toContain("dentist, house, work");
    expect(paper).toContain("health, home");
    expect(paper).not.toContain("list#1");
  });

  it("says which notices are on, not which are off", () => {
    const paper = written(seen, at);

    expect(paper).toMatch(/notices\s+A notification from the system/);
    expect(paper).not.toContain("A short tone");
  });

  it("says so when nothing would speak at all", () => {
    const paper = written({ ...seen, quiet: ["screen", "chime"] }, at);

    expect(paper).toMatch(/notices\s+none/);
  });

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

  it("keeps the backup in the report when a folder is shared, since the folder is no backup", () => {
    const paper = written({ ...seen, shared: true, backedUpAt: null }, at);

    expect(paper).toMatch(/backup\s+available/);
    expect(paper).toContain("none yet");
  });

  it("says when the last copy was made", () => {
    const paper = written({ ...seen, shared: false, backedUpAt: "2026-08-04T10:00:00Z" }, at);

    expect(paper).toMatch(/backup\s+available/);
    expect(paper).not.toContain("none yet");
  });

  it("names the cloud it syncs with rather than calling it this machine only", () => {
    const paper = written({ ...seen, shared: false, cloud: "Google Drive" }, at);

    expect(paper).toMatch(/syncs\s+yes, Google Drive/);
    expect(paper).not.toContain("this machine only");
  });

  it("says «none yet» where no copy has ever been made", () => {
    const paper = written({ ...seen, shared: false, backedUpAt: null }, at);

    expect(paper).toContain("none yet");
  });

  it("says loose was not checked when a document could not be read, rather than none", () => {
    const paper = written({ ...seen, loose: 0, looseBytes: 0, unread: true }, at);

    expect(paper).toContain("loose not checked: a document could not be read");
    expect(paper).not.toContain("0 loose");
  });

  it("says how this copy was installed and whether an assistant can file", () => {
    expect(written(seen, at)).toMatch(/installed from\s+a download/);
    expect(written({ ...seen, installed: "disk" }, at)).toContain("the disk image");
    expect(written(seen, at)).toMatch(/assistants\s+cannot file here/);
    expect(written({ ...seen, assistant: true }, at)).toMatch(/assistants\s+can file here/);
  });

  it("leaves out how large an attachment may be, which never changes", () => {
    expect(written(seen, at)).not.toContain("copies up to");
  });

  it("says on its face that nothing has been sent", () => {
    expect(written(seen, at)).toContain("none of this leaves your computer until you send it");
  });
});
