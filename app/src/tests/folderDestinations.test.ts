import { describe, expect, it } from "vitest";
import type { Folded } from "../core";
import { destinations } from "../folders";

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded =>
  ({ id, name, order: id, parent: null, ...more }) as Folded;

const agents = folder("ag", "AGENTS", { icon: "robot" });
const claude = folder("claude", "Claude", { parent: "ag" });
const codex = folder("codex", "Codex", { parent: "ag" });
const work = folder("work", "Trabajo");
const home = folder("home", "Casa");
const folders = [agents, claude, codex, work, home];

const offered = (moving: Folded) =>
  destinations(folders, moving.parent, () => {}, moving).map((one) => one.key);

describe("where a folder can be moved to", () => {
  it("never offers Agents or its folders for a plain folder", () => {
    expect(offered(work)).toEqual(["unfiled", "home"]);
  });

  it("offers them to a folder that is already one of Agents", () => {
    expect(offered(claude)).toContain("codex");
  });
});
