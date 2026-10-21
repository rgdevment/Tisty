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

describe("the report a bug gets attached to, as a whole", () => {
  it("reads the same on a fresh install", () => {
    expect(written(seen, at)).toMatchSnapshot();
  });

  it("reads the same with real names, a sandbox, a cloud and an assistant", () => {
    const paper = written(
      {
        ...seen,
        sandbox: "pruebas",
        listNames: ["dentist", "house", "work"],
        tagNames: ["health", "home"],
        shared: false,
        cloud: "Google Drive",
        backedUpAt: "2026-08-04T10:00:00",
        installed: "disk",
        assistant: true,
      },
      at,
    );
    expect(paper).toMatchSnapshot();
  });
});
