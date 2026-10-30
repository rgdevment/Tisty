import { frontLines } from "tisty-editor/read";
import { describe, expect, it } from "vitest";
import { fields, frontTags, overgrown, unfronted } from "../front";

describe("the properties at the top of a note", () => {
  it("are found only where a note opens with them, closed by a rule or three dots", () => {
    expect(frontLines("---\ntitle: x\n---\n\nhola")).toEqual(["---", "title: x", "---"]);
    expect(frontLines("﻿---\ntitle: x\n...\nhola")).toEqual(["---", "title: x", "..."]);
    expect(frontLines("---\n\ntitle: x\n---")).toBeNull();
    expect(frontLines("hola\n---\ntitle: x\n---")).toBeNull();
    expect(frontLines("---\ntitle: x")).toBeNull();
  });

  it("leave the rest of the note as it was when they are taken away", () => {
    expect(unfronted("---\ntitle: x\n---\n\n# Hola")).toBe("\n# Hola");
    expect(unfronted("# Hola\n\n---\nx\n---")).toBe("# Hola\n\n---\nx\n---");
  });

  it("are read the plain ways a note writes them", () => {
    const said = [
      "---",
      'title: "Plan de marzo"',
      "tags: [rina, 'vet']",
      "# a comment",
      "listed:",
      "  - uno",
      "- dos",
      "deep: x",
      "  more of it",
      "no colon here",
      "empty:",
      "---",
    ].join("\n");

    expect(fields(said)).toEqual([
      { key: "title", values: ["Plan de marzo"] },
      { key: "tags", values: ["rina", "vet"] },
      { key: "listed", values: ["uno", "dos"] },
      { key: "deep", values: ["x", "more of it"] },
      { key: "empty", values: [] },
    ]);
    expect(fields("# Hola")).toEqual([]);
  });

  it("are kept but never read once past their limits", () => {
    const keys = Array.from({ length: 33 }, (_, at) => `k${at}: v`).join("\n");
    const long = `nota: ${"a".repeat(501)}`;
    const big = `nota: ${"b ".repeat(2100)}`;

    expect(fields(`---\n${keys}\n---`)).toEqual([]);
    expect(fields(`---\n${long}\n---`)).toEqual([]);
    expect(overgrown(`---\n${big}\n---`)).toBe(true);
    expect(fields(`---\n${big}\n---`)).toEqual([]);
    expect(overgrown("---\ntitle: x\n---")).toBe(false);
    expect(overgrown("# Hola")).toBe(false);
  });

  it("file the note under every word its tags name, with or without the hash", () => {
    expect(frontTags("---\ntags: [rina, vet]\n---")).toEqual(["rina", "vet"]);
    expect(frontTags('---\ntag: "#uno #dos, tres"\n---')).toEqual(["uno", "dos", "tres"]);
    expect(frontTags("---\ntitle: x\n---")).toEqual([]);
  });
});
