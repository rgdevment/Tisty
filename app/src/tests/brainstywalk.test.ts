import { describe, expect, it, vi } from "vitest";
import type { Network, Neuron } from "../brainsty";
import type { Drawing } from "../ui/brainsty/drawing";
import { stepped, wayOf } from "../ui/brainsty/moving";
import { EVERYTHING, found, hiddenBy, holdersOf, sifting } from "../ui/brainsty/sifting";
import { walker } from "../ui/brainsty/walking";

const one = (id: string, kind: Neuron["kind"], state?: string, when?: string): Neuron => ({
  id,
  kind,
  title: id,
  state,
  when,
});

const network: Network = {
  nodes: [
    one("Plan de viaje", "doc", "open", "2026-11-10"),
    one("Pagar la luz", "task", "done", "2026-01-02"),
    one("Comprar pan", "task", "open"),
    one("viajes", "tag"),
    one("mapa.png", "file"),
    one("Índice", "page", "archived", "2026-11-17"),
  ],
  edges: [
    [0, 3, 3],
    [2, 3, 3],
    [0, 4, 4],
  ],
};

const today = new Date("2026-11-18T12:00:00Z");

describe("what the filters leave in sight", () => {
  it("keeps everything when nothing is asked", () => {
    expect(hiddenBy(network, EVERYTHING, today).every((gone) => !gone)).toBe(true);
    expect(sifting(EVERYTHING)).toBe(false);
  });

  it("leaves out the kinds, states and periods taken away, tags and files only by kind", () => {
    const sift = {
      ...EVERYTHING,
      kinds: new Set(["doc", "task", "page", "file"] as const),
      standings: new Set(["live"] as const),
      days: 30 as const,
    };

    expect(hiddenBy(network, sift, today)).toEqual([false, true, true, true, false, true]);
    expect(sifting(sift)).toBe(true);
  });

  it("keeps a tag and what carries it", () => {
    expect([...holdersOf(network, 3)].sort()).toEqual([0, 2, 3]);
    expect(hiddenBy(network, { ...EVERYTHING, tag: 3 }, today)).toEqual([
      false,
      true,
      false,
      false,
      true,
      true,
    ]);
  });

  it("finds titles by their start first, then a word, then anywhere, ignoring accents and what is hidden", () => {
    const none = network.nodes.map(() => false);

    expect(found(network, "pa", none)).toEqual([1, 2, 4]);
    expect(found(network, "indice", none)).toEqual([5]);
    expect(found(network, "viaj", none)).toEqual([3, 0]);
    expect(found(network, "viaj", [false, false, false, true, false, false])).toEqual([0]);
    expect(found(network, "  ", none)).toEqual([]);
  });
});

describe("walking the network with the keys", () => {
  const spots: Record<number, [number, number]> = {
    0: [0, 0],
    1: [100, 0],
    2: [30, 5],
    3: [0, 100],
  };
  const at = (node: number) => spots[node];

  it("reads the arrow keys", () => {
    expect(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "a"].map(wayOf)).toEqual([
      "up",
      "down",
      "left",
      "right",
      undefined,
    ]);
  });

  it("goes to a neighbour that way before the nearest node across the map", () => {
    expect(stepped(0, "right", at, [1], 4)).toBe(1);
    expect(stepped(0, "right", at, [3], 4)).toBe(2);
    expect(stepped(0, "down", at, [], 4)).toBe(3);
    expect(stepped(0, "up", at, [], 4)).toBeUndefined();
    expect(stepped(9, "up", at, [], 4)).toBeUndefined();
  });

  const press = (key: string, target: EventTarget = window) => {
    const event = new KeyboardEvent("keydown", { key, cancelable: true });
    Object.defineProperty(event, "target", { value: target });
    return event;
  };

  it("searches with /, opens with Enter and starts where it opened", () => {
    const drawing = {
      nodeOnScreen: at,
      beside: () => [1],
    } as unknown as Drawing;
    let picked: number | null = null;
    const walk = {
      picked: () => picked,
      hidden: () => [false, false, false, false],
      pick: vi.fn((node: number) => {
        picked = node;
      }),
      open: vi.fn(),
      find: vi.fn(),
    };
    const keyed = walker(drawing, 4, 0, walk);

    keyed(press("/"));
    keyed(press("Enter"));
    keyed(press("ArrowRight"));
    keyed(press("ArrowRight"));
    keyed(press("Enter"));
    keyed(press("ArrowLeft", document.createElement("input")));
    keyed(press("Enter", document.createElement("button")));
    keyed(press("x"));

    expect(walk.find).toHaveBeenCalledTimes(1);
    expect(walk.pick.mock.calls).toEqual([[0], [1]]);
    expect(walk.open).toHaveBeenCalledWith(1);
  });

  it("starts on the first node in sight when where it opened is hidden", () => {
    const pick = vi.fn();
    const keyed = walker({} as Drawing, 3, 0, {
      picked: () => null,
      hidden: () => [true, false, false],
      pick,
      open: vi.fn(),
      find: vi.fn(),
    });

    keyed(press("ArrowDown"));

    expect(pick).toHaveBeenCalledWith(1);
  });
});
