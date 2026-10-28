/** Shapes with brackets of their own, written `A[…]`, `A{…}` and the like. */
export type Bracketed =
  | "process"
  | "rounded"
  | "stadium"
  | "subroutine"
  | "database"
  | "circle"
  | "double"
  | "decision"
  | "hexagon"
  | "lean"
  | "leanLeft"
  | "trapezoid"
  | "trapezoidDown"
  | "flag";

// Mermaid's newer shapes, by the name each is written with; they exist only as `A@{ shape: … }`.
export const NAMED = [
  "datastore",
  "folder",
  "bucket",
  "console",
  "browser",
  "person",
  "bang",
  "cloud",
  "text",
  "notch-rect",
  "lin-rect",
  "sm-circ",
  "fr-circ",
  "fork",
  "hourglass",
  "brace",
  "brace-r",
  "braces",
  "bolt",
  "doc",
  "delay",
  "h-cyl",
  "lin-cyl",
  "curv-trap",
  "div-rect",
  "tri",
  "win-pane",
  "f-circ",
  "notch-pent",
  "flip-tri",
  "sl-rect",
  "docs",
  "st-rect",
  "bow-rect",
  "cross-circ",
  "tag-doc",
  "tag-rect",
  "paper-tape",
  "lin-doc",
] as const;

export type Shape = Bracketed | (typeof NAMED)[number];

// Each shape, then every name Mermaid 12 takes for it in `@{ shape: … }`, the one written first.
// Mermaid's own `flag` is the paper tape; the window's flag is Mermaid's `odd`, the `A>…]` shape.
const NAMES = `
process rect proc process rectangle
rounded rounded event
stadium stadium terminal pill
subroutine fr-rect subprocess subproc framed-rectangle subroutine
database cyl db database cylinder
circle circle circ
double dbl-circ double-circle
decision diam decision diamond question
hexagon hex hexagon prepare
lean lean-r lean-right in-out
leanLeft lean-l lean-left out-in
trapezoid trap-b priority trapezoid-bottom trapezoid
trapezoidDown trap-t manual trapezoid-top inv-trapezoid
flag odd
datastore datastore data-store
folder folder directory
bucket bucket
console console
browser browser
person person
bang bang
cloud cloud
text text
notch-rect notch-rect card notched-rectangle
lin-rect lin-rect lined-rectangle lined-process lin-proc shaded-process
sm-circ sm-circ start small-circle
fr-circ fr-circ stop framed-circle
fork fork join
hourglass hourglass collate
brace brace comment brace-l
brace-r brace-r
braces braces
bolt bolt com-link lightning-bolt
doc doc document
delay delay half-rounded-rectangle
h-cyl h-cyl das horizontal-cylinder
lin-cyl lin-cyl disk lined-cylinder
curv-trap curv-trap curved-trapezoid display
div-rect div-rect div-proc divided-rectangle divided-process
tri tri extract triangle
win-pane win-pane internal-storage window-pane
f-circ f-circ junction filled-circle
notch-pent notch-pent loop-limit notched-pentagon
flip-tri flip-tri manual-file flipped-triangle
sl-rect sl-rect manual-input sloped-rectangle
docs docs documents st-doc stacked-document
st-rect st-rect procs processes stacked-rectangle
bow-rect bow-rect stored-data bow-tie-rectangle
cross-circ cross-circ summary crossed-circle
tag-doc tag-doc tagged-document
tag-rect tag-rect tagged-rectangle tag-proc tagged-process
paper-tape paper-tape flag
lin-doc lin-doc lined-document
`
  .trim()
  .split("\n")
  .map((line) => line.split(" "));

const BY_NAME = new Map(NAMES.flatMap(([shape, ...names]) => names.map((one) => [one, shape])));
const WRITTEN = new Map(NAMES.map(([shape, name]) => [shape, name]));

/** The shape a name in `@{ shape: … }` stands for, or `null` when Mermaid would not draw it. */
export const shapeNamed = (name: string): Shape | null =>
  (BY_NAME.get(name) as Shape | undefined) ?? null;

/** The name Mermaid is given for a shape inside `@{ … }`. */
export const nameOfShape = (shape: Shape): string => WRITTEN.get(shape) ?? shape;

export const bracketed = (shape: Shape): shape is Bracketed => !NAMED.some((one) => one === shape);

/** What `@{ … }` says, key by key, each value as written. */
export type Keys = readonly (readonly [string, string])[];

const KEY = /\s*([A-Za-z][\w-]*)\s*:\s*/y;
const BARE = /[^,}"]*/y;

/** The keys of the `@{ … }` that starts at `from`, and where it ends; `null` for what the window cannot read. */
export const keysAt = (said: string, from: number): { keys: Keys; end: number } | null => {
  if (!said.startsWith("@{", from)) return null;
  const keys: [string, string][] = [];
  let here = from + 2;
  for (;;) {
    KEY.lastIndex = here;
    const key = KEY.exec(said);
    if (!key) break;
    here = KEY.lastIndex;
    let value: string;
    if (said[here] === '"') {
      const close = said.indexOf('"', here + 1);
      if (close < 0) return null;
      value = said.slice(here, close + 1);
      here = close + 1;
    } else {
      BARE.lastIndex = here;
      value = (BARE.exec(said)?.[0] ?? "").trim();
      here = BARE.lastIndex;
    }
    if (!value) return null;
    keys.push([key[1], value]);
    while (said[here] === " ") here += 1;
    if (said[here] !== ",") break;
    here += 1;
  }
  while (said[here] === " ") here += 1;
  return said[here] === "}" ? { keys, end: here + 1 } : null;
};

export const keyValue = (keys: Keys, key: string): string | null =>
  keys.find(([one]) => one === key)?.[1] ?? null;

// A key already said is rewritten where it stands; a new one goes last.
const withKey = (keys: Keys, key: string, value: string): Keys =>
  keys.some(([one]) => one === key)
    ? keys.map(([one, was]) => [one, one === key ? value : was] as const)
    : [...keys, [key, value] as const];

const OPENING: Record<Bracketed, readonly [string, string]> = {
  process: ["[", "]"],
  rounded: ["(", ")"],
  stadium: ["([", "])"],
  subroutine: ["[[", "]]"],
  database: ["[(", ")]"],
  circle: ["((", "))"],
  double: ["(((", ")))"],
  decision: ["{", "}"],
  hexagon: ["{{", "}}"],
  lean: ["[/", "/]"],
  leanLeft: ["[\\", "\\]"],
  trapezoid: ["[/", "\\]"],
  trapezoidDown: ["[\\", "/]"],
  flag: [">", "]"],
};

export const unquoted = (value: string) =>
  (value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value).replace(
    /#quot;/g,
    '"',
  );

/** The words a label in `@{ … }` says. */
export const labelOf = (keys: Keys): string | null => {
  const said = keyValue(keys, "label");
  return said === null ? null : unquoted(said);
};

// Only what changed is rewritten, so a name like `diamond` or a bare label stays as it was typed.
const rekeyed = (keys: Keys, id: string, text: string, shape: Shape | null): Keys => {
  const named = keyValue(keys, "shape");
  const shaped =
    shape === null || (named !== null && shapeNamed(named) === shape)
      ? keys
      : withKey(keys, "shape", nameOfShape(shape));
  return (labelOf(keys) ?? id) === text
    ? shaped
    : withKey(shaped, "label", `"${text.replace(/"/g, "#quot;")}"`);
};

/**
 * A node as Mermaid reads it back: in brackets while its shape has them and it was not written
 * with `@{ … }`, whose other keys then stay as they were.
 */
export const formedNode = (
  id: string,
  text: string,
  shape: Shape | null,
  keys: Keys | null = null,
): string => {
  const drawn = shape ?? "process";
  if (keys === null && bracketed(drawn)) {
    const [open, close] = OPENING[drawn];
    const plain = text.trim() === text && !/["()[\]{}|<>;]/.test(text);
    return `${id}${open}${plain ? text : `"${text.replace(/"/g, "#quot;")}"`}${close}`;
  }
  const said = rekeyed(keys ?? [], id, text, keys === null ? drawn : shape);
  return `${id}@{ ${said.map(([key, value]) => `${key}: ${value}`).join(", ")} }`;
};
