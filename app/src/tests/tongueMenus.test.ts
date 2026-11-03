import { describe, expect, it, vi } from "vitest";
import type { Filed, Folded } from "../core";
import type { Hands } from "../docMenus";
import { fill } from "../locales";
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

  it("asks each document's language by its name, never as this one or that one", () => {
    const en = { ...doc("Network", "rsrc"), lang: null };
    const es = { ...doc("Red", "rsrc"), lang: null };
    const hands = {
      papers: { docs: [en, es], folders: [folder("rsrc", null)] },
    } as unknown as Hands;

    const join = tongueChoices(en, hands).find((one) => one.key === "joinTongues");
    const red = join?.into?.choices[0];
    const english = red?.into?.choices.find((one) => one.key === "en");

    expect(red?.into?.label).toBe(fill("tongueOf", "Network"));
    expect(english?.into?.label).toBe(fill("tongueOf", "Red"));
  });
});
