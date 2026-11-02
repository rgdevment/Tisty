import { describe, expect, it, vi } from "vitest";
import type { Filed, Folded } from "../core";
import type { Hands } from "../docMenus";
import { tongueChoices } from "../tongueMenus";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve() }));

const doc = (id: string, folder: string | null): Filed => ({
  id,
  file: `f-${id}`,
  title: id,
  folder,
  archived: false,
  away: false,
  lang: "es",
});

const folder = (id: string, parent: string | null): Folded =>
  ({ id, name: id, parent, holds: 0, archived: false, away: false }) as Folded;

describe("joining as a translation", () => {
  it("offers the documents beside it and one folder up, not every folder below that", () => {
    const docs = [
      doc("rsrc-en", "rsrc"),
      doc("rsrc-es", "rsrc"),
      doc("projects-doc", "projects"),
      doc("sibling-doc", "tisty"),
      doc("loose", null),
    ];
    const hands = {
      papers: {
        docs,
        folders: [
          folder("projects", null),
          folder("rsrc", "projects"),
          folder("tisty", "projects"),
        ],
      },
    } as unknown as Hands;

    const join = tongueChoices(docs[0], hands).find((one) => one.key === "joinTongues");

    expect(join?.into?.choices.map((one) => one.key)).toEqual(["rsrc-es", "projects-doc"]);
  });
});
