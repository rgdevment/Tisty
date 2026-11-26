import { type Asked, pressed, SLASHED, type Slashed } from "tisty-editor/edit";
import { t, type Word } from "../locales";
import type { Choice } from "./Menu";

/** What the page asking for the menu holds and allows, which decides the rows it is offered. */
export type InsertAsked = {
  /** The kinds of thing the document's folder lets be asked for, as its host says. */
  readonly kinds: readonly Asked[];
  /** The page's own steps, in order, which a step is named by. */
  readonly steps: readonly string[];
  /** A task's page, whose headings are its sections, each made from its own button. */
  readonly sections: boolean;
  readonly pick: (one: Slashed) => void;
};

type Group = "text" | "lists" | "insert" | "link";

/** A row of the menu: one block, or a list of them opened beside it. */
type Entry = {
  readonly key: string;
  readonly group: Group;
  /** The rows inside: blocks by their key, or the page's steps. */
  readonly into?: readonly string[] | "steps";
  readonly label?: Word;
  readonly glyph?: string;
  readonly mark?: string;
  readonly keys?: string;
};

const GROUPS: Record<Group, Word> = {
  text: "insertGroupText",
  lists: "insertGroupLists",
  insert: "insertGroupInsert",
  link: "insertGroupLink",
};

const PARAGRAPH: Slashed = {
  key: "paragraph",
  label: "paragraph",
  hint: "",
  icon: "pilcrow",
  put: { line: "" },
};

const ENTRIES: readonly Entry[] = [
  { key: "paragraph", group: "text" },
  { key: "h1", group: "text", mark: "#" },
  { key: "h2", group: "text", mark: "##" },
  { key: "h3", group: "text", mark: "###" },
  { key: "quote", group: "text", mark: ">" },
  {
    key: "callout",
    group: "text",
    label: "insertCallout",
    glyph: "info",
    into: [
      "callout-note",
      "callout-tip",
      "callout-important",
      "callout-warning",
      "callout-caution",
      "callout-idea",
    ],
  },
  { key: "bullets", group: "lists", mark: "-" },
  { key: "numbers", group: "lists", mark: "1." },
  { key: "todo", group: "lists", mark: "[ ]" },
  { key: "table", group: "insert" },
  { key: "file", group: "insert", label: "insertPicture", glyph: "picture" },
  { key: "code", group: "insert", mark: "```" },
  {
    key: "diagram",
    group: "insert",
    label: "insertDiagram",
    glyph: "graph",
    into: ["diagram", "diagramFrom", "mermaid"],
  },
  { key: "math", group: "insert" },
  { key: "widget", group: "insert" },
  { key: "fold", group: "insert" },
  { key: "rule", group: "insert", glyph: "pagebreak", keys: pressed("Enter") },
  { key: "newpage", group: "link" },
  {
    key: "pageOrDoc",
    group: "link",
    label: "insertPageOrDoc",
    glyph: "page",
    into: ["page", "doc", "newdoc"],
  },
  { key: "task", group: "link", label: "insertTaskOf", glyph: "todo", into: ["newtask", "task"] },
  { key: "step", group: "link", label: "insertStepOf", glyph: "checks", into: "steps" },
  { key: "icon", group: "link" },
];

/** The shortcuts over the list, while nothing is sought: the blocks most often wanted. */
const TILES: readonly { key: string; label: Word; glyph: string }[] = [
  { key: "h1", label: "bigTitle", glyph: "heading1" },
  { key: "bullets", label: "insertTileList", glyph: "bullets" },
  { key: "todo", label: "insertTileChecks", glyph: "checks" },
  { key: "table", label: "table", glyph: "table" },
  { key: "file", label: "insertTilePicture", glyph: "picture" },
  { key: "code", label: "insertTileCode", glyph: "code" },
];

const blockOf = (key: string): Slashed | undefined =>
  key === PARAGRAPH.key ? PARAGRAPH : SLASHED.find((one) => one.key === key);

// More words each row is found by, one line in each language: `key: words; key: words`.
const wordsOf = (key: string): string | undefined =>
  t("insertWords")
    .split(";")
    .map((pair) => pair.split(":"))
    .find(([named]) => named.trim() === key)?.[1]
    ?.trim();

const offeredHere = (one: Slashed, asked: InsertAsked) =>
  !("wrap" in one.put) &&
  (!("ask" in one.put) || asked.kinds.includes(one.put.ask)) &&
  !(asked.sections && /^h\d$/.test(one.key));

const leafOf = (one: Slashed, asked: InsertAsked): Choice => ({
  key: one.key,
  label: t(one.label),
  glyph: one.icon,
  words: wordsOf(one.key),
  onPick: () => asked.pick(one),
});

// A step is named by its number, so it is chosen from the page's own steps.
const stepsOf = (asked: InsertAsked): Choice[] => {
  const step = blockOf("step");
  if (!step || asked.steps.length === 0) return [];
  return asked.steps.map((words, at) => {
    const named = `[[#${at + 1}]]`;
    return {
      key: named,
      label: `${at + 1}. ${words}`,
      onPick: () => asked.pick({ ...step, put: { inline: named, at: named.length } }),
    };
  });
};

const insideOf = (entry: Entry, asked: InsertAsked): Choice[] => {
  if (entry.into === "steps") return stepsOf(asked);
  return (entry.into ?? [])
    .map(blockOf)
    .filter((one): one is Slashed => one !== undefined && offeredHere(one, asked))
    .map((one) => leafOf(one, asked));
};

const rowOf = (entry: Entry, asked: InsertAsked): Choice | null => {
  const dressed = {
    group: t(GROUPS[entry.group]),
    words: wordsOf(entry.key),
    mark: entry.mark,
    keys: entry.keys,
  };
  if (!entry.into) {
    const one = blockOf(entry.key);
    if (!one || !offeredHere(one, asked)) return null;
    const leaf = leafOf(one, asked);
    if (!entry.label) return { ...leaf, glyph: entry.glyph ?? one.icon, ...dressed };
    // Renamed in the menu, it is still found by the name the block goes by.
    const words = [dressed.words, leaf.label].filter(Boolean).join(" ");
    return { ...leaf, label: t(entry.label), glyph: entry.glyph ?? one.icon, ...dressed, words };
  }
  const inside = insideOf(entry, asked);
  if (inside.length === 0) return null;
  const label = entry.label ? t(entry.label) : "";
  // A list left with one row is that row, still found by the words of the list it stood in.
  if (inside.length === 1 && entry.into !== "steps")
    return { ...inside[0], key: entry.key, glyph: entry.glyph, ...dressed };
  return {
    key: entry.key,
    label,
    glyph: entry.glyph,
    ...dressed,
    into: { label, choices: inside },
  };
};

/** The rows of the Insert menu and its shortcuts, as the page asking for it allows them. */
export const insertChoices = (asked: InsertAsked): { choices: Choice[]; row: Choice[] } => {
  const choices = ENTRIES.map((entry) => rowOf(entry, asked)).filter(
    (one): one is Choice => one !== null,
  );
  const row = TILES.flatMap((tile) => {
    const one = blockOf(tile.key);
    if (!one || !offeredHere(one, asked)) return [];
    return [
      { key: tile.key, label: t(tile.label), glyph: tile.glyph, onPick: () => asked.pick(one) },
    ];
  });
  return { choices, row };
};
