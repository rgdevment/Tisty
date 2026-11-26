import { FORMATS, pressed } from "tisty-editor/edit";
import { fill, t } from "../locales";
import type { Choice } from "./Menu";
import { MOVE_KEYS } from "./macKeys";
import type { Act } from "./pageMoving";

export type Spelled = { readonly wrong: boolean; readonly guesses: readonly string[] };

export type PageMenuAsked = {
  readonly words: string | null;
  readonly cuttable: boolean;
  readonly said: string | null;
  readonly spelled: Spelled | null;
  readonly formattable: boolean;
  /** Agents' files take only what they read as: words in and out, no blocks or formats. */
  readonly limited: boolean;
  readonly block: boolean;
  readonly pasting: boolean;
};

export type PageMenuActs = {
  readonly guessed: (guess: string) => void;
  readonly learned: () => void;
  readonly cut: () => void;
  readonly copy: () => void;
  readonly paste: (plain: boolean) => void;
  readonly format: (mark: string, close: string) => void;
  readonly insert: () => void;
  readonly blockActed: (act: Act) => void;
};

const spellingOf = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] => {
  const { said, spelled } = asked;
  if (!said || !spelled?.wrong) return [];
  const guesses: Choice[] = spelled.guesses.map((guess) => ({
    key: `guess-${guess}`,
    label: guess,
    onPick: () => acts.guessed(guess),
  }));
  return [
    ...(guesses.length > 0 ? guesses : [{ key: "guess-none", label: t("spellNone"), off: true }]),
    { key: "learn", label: fill("spellLearn", said), onPick: acts.learned },
  ];
};

const pastesOf = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] =>
  asked.pasting
    ? [
        { key: "paste", label: t("pasteIt"), keys: pressed("V"), onPick: () => acts.paste(false) },
        {
          key: "paste-plain",
          label: t("pastePlain"),
          keys: pressed("Shift V"),
          onPick: () => acts.paste(true),
        },
      ]
    : [];

const clipboardOf = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] => [
  ...(asked.words
    ? [
        ...(asked.cuttable
          ? [{ key: "cut", label: t("cutIt"), keys: pressed("X"), onPick: acts.cut }]
          : []),
        { key: "copy", label: t("copyIt"), keys: pressed("C"), onPick: acts.copy },
      ]
    : []),
  ...pastesOf(asked, acts),
];

const formatsOf = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] =>
  asked.words && asked.formattable && !asked.limited
    ? FORMATS.map(([word, keys, mark, close]) => ({
        key: `format-${word}`,
        label: t(word),
        keys: pressed(keys),
        onPick: () => acts.format(mark, close),
      }))
    : [];

const blocksOf = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] => {
  if (asked.words || !asked.block || asked.limited) return [];
  const acted = (act: Act) => () => acts.blockActed(act);
  return [
    { key: "insert", label: t("insertOf"), onPick: acts.insert },
    { key: "block-up", label: t("blockUp"), keys: `${MOVE_KEYS} ↑`, onPick: acted("up") },
    { key: "block-down", label: t("blockDown"), keys: `${MOVE_KEYS} ↓`, onPick: acted("down") },
    { key: "block-double", label: t("blockDouble"), onPick: acted("double") },
    { key: "block-drop", label: t("blockDrop"), danger: true, apart: true, onPick: acted("drop") },
  ];
};

export const pageChoices = (asked: PageMenuAsked, acts: PageMenuActs): Choice[] =>
  [spellingOf, clipboardOf, formatsOf, blocksOf]
    .map((group) => group(asked, acts))
    .filter((group) => group.length > 0)
    .flatMap((group, at) =>
      group.map((one, index) => (index === 0 ? { ...one, apart: at > 0 } : one)),
    );
