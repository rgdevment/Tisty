import { beforeEach, describe, expect, it } from "vitest";
import { allParts, partsIn } from "../agentsAmiss";
import { noticeSaid, partsSaid, pathsSaid, pendingSaid, placeNamed } from "../agentsSaid";
import type { Wired } from "../core";
import { adopt } from "../locales";
import { lostOne, strayHooks, wiredOne } from "./knownFor";

beforeEach(() => adopt("es"));

const connection = (name: string, gone = true) => lostOne(name, { gone });
const hooks = (name: string, gone = true) =>
  wiredOne(name, { wired: true, hooks: strayHooks("/viejo/tisty", gone) });

describe("what is said of an agent that needs something", () => {
  it.each<[string, Wired[], string, string]>([
    [
      "one connection",
      [connection("Codex")],
      "Codex ya no encuentra Tisty",
      "Codex apunta a un Tisty que ya no existe",
    ],
    [
      "one's hooks",
      [hooks("Codex")],
      "Los hooks de Codex ya no encuentran Tisty",
      "Los hooks de Codex apuntan a un Tisty que ya no existe",
    ],
    [
      "connections",
      [connection("A"), connection("B")],
      "2 agentes ya no encuentran Tisty",
      "2 agentes apuntan a un Tisty que ya no existe",
    ],
    [
      "hooks",
      [hooks("A"), hooks("B")],
      "Los hooks de 2 agentes ya no encuentran Tisty",
      "Los hooks de 2 agentes apuntan a un Tisty que ya no existe",
    ],
    [
      "both",
      [connection("A"), hooks("B")],
      "2 agentes, o sus hooks, ya no encuentran Tisty",
      "2 agentes, o sus hooks, apuntan a un Tisty que ya no existe",
    ],
  ])("names what was lost and nothing else: %s", (_, all, notice, pending) => {
    expect(noticeSaid(all)).toBe(notice);
    expect(pendingSaid(all, "lost")).toBe(pending);
  });

  it.each<[Wired[], string]>([
    [[connection("Codex", false)], "Codex usa tu otra instalación de Tisty"],
    [[hooks("Codex", false)], "Los hooks de Codex usan tu otra instalación de Tisty"],
    [
      [connection("A", false), connection("B", false)],
      "2 agentes usan tu otra instalación de Tisty",
    ],
    [
      [hooks("A", false), hooks("B", false)],
      "Los hooks de 2 agentes usan tu otra instalación de Tisty",
    ],
    [
      [connection("A", false), hooks("B", false)],
      "2 agentes, o sus hooks, usan tu otra instalación de Tisty",
    ],
  ])("tells another Tisty apart from one that is gone", (all, said) => {
    expect(pendingSaid(all, "elsewhere")).toBe(said);
  });

  it("says of a connection to another Tisty only that its hooks were lost, when only they were", () => {
    const one = connection("Codex", false);
    const mixed = { ...one, points: "/otro/tisty", hooks: strayHooks("/viejo/tisty") };

    expect(noticeSaid([mixed])).toBe("Los hooks de Codex ya no encuentran Tisty");
    expect(partsSaid(partsIn(mixed, "lost"))).toBe("hooks");
    expect(pathsSaid(mixed, partsIn(mixed, "lost"))).toEqual([
      "Sus hooks apuntan a /viejo/tisty, que ya no existe",
    ]);
    expect(partsSaid(partsIn(mixed, "elsewhere"))).toBe("conexión");
    expect(partsSaid(allParts(mixed))).toBe("conexión y hooks");
    expect(pathsSaid(mixed, allParts(mixed))).toEqual([
      "Apunta a /otro/tisty, que no es este Tisty",
      "Sus hooks apuntan a /viejo/tisty, que ya no existe",
    ]);
  });
});

describe("what is said of an agent inside a WSL distro", () => {
  const inUbuntu = lostOne("wsl:Ubuntu:claude-code", { name: "Claude Code", place: "Ubuntu" });

  it("names the distro in the row's name, and only for a row that has one", () => {
    expect(placeNamed(inUbuntu)).toBe("Claude Code (Ubuntu · WSL)");
    expect(placeNamed(lostOne("claude-code", { name: "Claude Code" }))).toBe("Claude Code");
  });

  it("names it in the notice and in the pending line when it is the only one", () => {
    expect(noticeSaid([inUbuntu])).toBe("Claude Code (Ubuntu · WSL) ya no encuentra Tisty");
    expect(pendingSaid([inUbuntu], "lost")).toBe(
      "Claude Code (Ubuntu · WSL) apunta a un Tisty que ya no existe",
    );
  });
});
