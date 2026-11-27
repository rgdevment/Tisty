import { describe, expect, it, vi } from "vitest";
import type { Filed, Snapshot } from "../core";
import { deskOf } from "../desk";
import type { Choice } from "../ui/Menu";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));

const one: Filed = {
  id: "D1",
  file: "f-D1",
  title: "Uno",
  folder: null,
  archived: false,
  away: false,
};

const stand = () => {
  const given: Record<string, unknown> = {
    tight: false,
    error: null,
    selected: undefined,
    mode: "side",
    found: null,
    asideShut: false,
    afoot: false,
    showing: null,
    held: undefined,
    greet: false,
    leaving: false,
    torn: false,
    asked: false,
    asking: false,
    marking: false,
    chosen: { named: "docs" },
    carries: { current: null },
    papers: { folders: [], docs: [one] },
    board: null,
    acted: { current: null },
    parcels: { asking: false },
    setMenu: vi.fn(),
  };
  const spies = new Map<string, ReturnType<typeof vi.fn>>();
  return new Proxy(given, {
    get: (target, key: string) => {
      if (key in target) return target[key];
      if (!spies.has(key)) spies.set(key, vi.fn());
      return spies.get(key);
    },
  });
};

const opened = (first?: Choice[]) => {
  const held = stand();
  const desk = deskOf(
    held as unknown as Parameters<typeof deskOf>[0],
    { tasks: [] } as unknown as Snapshot,
  );
  desk.docMenu(one, { x: 1, y: 2 }, first);
  return (held.setMenu as ReturnType<typeof vi.fn>).mock.calls[0][0] as {
    choices: Choice[];
    on: string;
  };
};

describe("the menu of a document", () => {
  it("opens on its own choices alone when given nothing first", () => {
    const menu = opened();

    expect(menu.on).toBe("D1");
    expect(menu.choices.some((one) => one.key === "pick")).toBe(false);
  });

  it("puts what it is given at the head, and sets its own first choice apart", () => {
    const alone = opened().choices;
    const pick: Choice = { key: "pick", icon: "☐", label: "Elegir", onPick: vi.fn() };

    const menu = opened([pick]);

    expect(menu.choices[0]).toBe(pick);
    expect(menu.choices).toHaveLength(alone.length + 1);
    expect(menu.choices[1].key).toBe(alone[0].key);
    expect(menu.choices[1].apart).toBe(true);
    expect(menu.choices[2].apart).toBe(alone[1].apart);
  });
});
