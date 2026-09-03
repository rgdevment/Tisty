export type Kind = "doc" | "folder";
export type Where = "before" | "in" | "after";

export const DEEPEST = 4;

export interface Spot {
  id: string;
  kind: Kind;
  parent?: string;
  next?: string;
  holds: boolean;
  line: string[];
  depth: number;
}

export interface Carried {
  id: string;
  kind: Kind;
  tall: number;
}

export interface Move {
  kind: Kind;
  moved: string;
  folder?: string;
  before?: string;
  pageOf?: string;
}

export const zoneIn = (top: number, height: number, y: number, thirds: boolean): Where => {
  if (!height) return "in";
  const at = (y - top) / height;
  if (!thirds) return at < 0.5 ? "before" : "after";
  if (at < 1 / 3) return "before";
  if (at > 2 / 3) return "after";
  return "in";
};

export const marked = (spot: Spot, where: Where) =>
  where === "in" ? spot.id : `${spot.id}:${where}`;

export const LOOSE = "~loose";

export const fits = (carried: Carried, into: Spot, where: Where) => {
  if (carried.kind !== "folder") return true;
  const under = where === "in" ? [...into.line, into.id] : into.line;
  if (under.includes(carried.id)) return false;
  const deep = where === "in" ? into.depth + 1 : into.depth;
  return deep + carried.tall <= DEEPEST;
};

export const settled = (carried: Carried, spot: Spot | null, where: Where): Move | null => {
  if (!spot) return null;
  if (carried.id === spot.id) return null;
  if (spot.id === LOOSE) return { kind: carried.kind, moved: carried.id };
  if (!fits(carried, spot, where)) return null;

  if (carried.kind === "folder") {
    if (spot.kind === "doc") return { kind: "folder", moved: carried.id, folder: spot.parent };
    if (where === "in") return { kind: "folder", moved: carried.id, folder: spot.id };
    const before = where === "before" ? spot.id : spot.next;
    if (before === carried.id) return null;
    return { kind: "folder", moved: carried.id, folder: spot.parent, before };
  }

  if (spot.kind === "folder") {
    if (where === "in") return { kind: "doc", moved: carried.id, folder: spot.id };
    return { kind: "doc", moved: carried.id, folder: spot.parent };
  }

  if (where === "in") {
    return spot.holds ? { kind: "doc", moved: carried.id, pageOf: spot.id } : null;
  }
  const before = where === "before" ? spot.id : spot.next;
  if (before === carried.id) return null;
  return { kind: "doc", moved: carried.id, folder: spot.parent, before };
};

export const BAND = 56;
export const MOST = 16;

export const speedAt = (top: number, bottom: number, y: number, band = BAND, most = MOST) => {
  if (bottom - top < band * 2) return 0;
  const paced = (over: number) => Math.ceil(Math.min(over / band, 1) * most);
  if (y < top + band) return -paced(top + band - y);
  if (y > bottom - band) return paced(y - (bottom - band));
  return 0;
};
