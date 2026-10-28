/** Where the window puts a shape, or a lane and its size, kept as a `%% tisty:` line after the body. */
export interface Place {
  readonly id: string;
  readonly x: number;
  readonly y: number;
  readonly wide?: number;
  readonly tall?: number;
  readonly line: string;
}

export type Spot = {
  readonly x: number;
  readonly y: number;
  readonly wide?: number;
  readonly tall?: number;
};

// A `namespace` of a class diagram may have dots in its name, as `com.zoo`.
const PLACE =
  /^\s*%% tisty: ([\p{L}\p{N}_.]+(?:-[\p{L}\p{N}_.]+)*) (-?\d+) (-?\d+)(?: (\d+) (\d+))?\s*$/u;

const placeOf = (line: string): Place | null => {
  const found = PLACE.exec(line);
  if (!found) return null;
  const [, id, x, y, wide, tall] = found;
  const sized = wide ? { wide: Number(wide), tall: Number(tall) } : {};
  return { id, x: Number(x), y: Number(y), ...sized, line };
};

/** A diagram's lines told apart: what Mermaid reads, and the window's places. */
export const placesApart = (lines: readonly string[]) => {
  const body: string[] = [];
  const places: Place[] = [];
  for (const line of lines) {
    const place = placeOf(line);
    if (place) places.push(place);
    else body.push(line);
  }
  return { body, places };
};

// A place for something no longer there, or said again, goes with the next change; the last one said wins.
export const tidied = (places: readonly Place[], there: ReadonlySet<string>): Place[] => {
  const last = new Map<string, Place>();
  for (const place of places) if (there.has(place.id)) last.set(place.id, place);
  return places.filter((one) => last.get(one.id) === one);
};

const indentOf = (line: string) => line.slice(0, line.length - line.trimStart().length);

/** The places with one written anew: its own line keeps its indent, so only the numbers change. */
export const placed = (
  places: readonly Place[],
  indent: string,
  id: string,
  at: Spot,
): readonly Place[] => {
  const known = places.find((one) => one.id === id);
  const wide = at.wide ?? known?.wide;
  const tall = at.tall ?? known?.tall;
  const spot = {
    x: Math.round(at.x),
    y: Math.round(at.y),
    ...(wide !== undefined && tall !== undefined
      ? { wide: Math.round(wide), tall: Math.round(tall) }
      : {}),
  };
  const line = `${known ? indentOf(known.line) : indent}%% tisty: ${id} ${Object.values(spot).join(" ")}`;
  const place = { id, ...spot, line };
  return known ? places.map((one) => (one.id === id ? place : one)) : [...places, place];
};

/** A diagram read again, handed back as a change: its body, its places and what is chosen in it. */
export const settledInto = <Picked>(
  diagram: { readonly body: readonly string[]; readonly places: readonly Place[] },
  picked: Picked,
) => ({ body: diagram.body, places: diagram.places, picked });
