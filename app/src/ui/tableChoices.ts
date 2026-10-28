import { fill, t } from "../locales";
import type { Choice } from "./Menu";
import type { Filtering } from "./tabled";

/** A column as its head says it: its title, and the words of its type, total and currency, empty when it has none. */
export type Headed = {
  readonly name: string;
  readonly type: string;
  readonly total: string;
  readonly currency: string;
};

export type ColumnWays = {
  readonly sort: (down: boolean) => void;
  readonly filter: (one: Omit<Filtering, "column">) => void;
  readonly asking: () => void;
  readonly retype: (words: string) => void;
  readonly rename: () => void;
  readonly insert: (right: boolean) => void;
  readonly remove: () => void;
  readonly markdown: () => void;
};

type Word = Parameters<typeof t>[0];

export const TYPES: readonly [string, Word][] = [
  ["text", "tableTypeText"],
  ["number", "tableTypeNumber"],
  ["money", "tableTypeMoney"],
  ["percent", "tableTypePercent"],
  ["date", "tableTypeDate"],
  ["duration", "tableTypeDuration"],
  ["check", "tableTypeCheck"],
  ["select", "tableTypeSelect"],
  ["tags", "tableTypeTags"],
  ["rating", "tableTypeRating"],
  ["link", "tableTypeLink"],
  ["url", "tableTypeUrl"],
  ["email", "tableTypeEmail"],
  ["phone", "tableTypePhone"],
];

const TOTALS: readonly [string, Word][] = [
  ["sum", "tableTotalSum"],
  ["avg", "tableTotalAvg"],
  ["min", "tableTotalMin"],
  ["max", "tableTotalMax"],
  ["count", "tableTotalCount"],
];

// The currencies Tisty writes as they are written where they are spent; any other code still works by hand.
const CURRENCIES = [
  "CLP",
  "USD",
  "EUR",
  "ARS",
  "MXN",
  "COP",
  "PEN",
  "BRL",
  "UYU",
  "CAD",
  "AUD",
  "GBP",
  "CHF",
  "JPY",
  "CNY",
];

const MEASURED = new Set(["number", "money", "percent", "duration", "rating"]);

// The same rule the core reads a head by: count goes anywhere, sums only where there is something to add.
const addsUp = (type: string, total: string): boolean =>
  total === "count" ||
  (MEASURED.has(type) && total !== "") ||
  (type === "date" && (total === "min" || total === "max"));

/** The words in a head's braces; a text column with nothing to add up has none. */
export const wordsOf = (type: string, currency: string, total: string): string => {
  const kept = type || "text";
  if (kept === "text" && !total) return "";
  return [kept, kept === "money" ? currency : "", total].filter(Boolean).join(" ");
};

const SORTS: Record<string, [Word, Word]> = {
  number: ["tableSortLow", "tableSortHigh"],
  money: ["tableSortLow", "tableSortHigh"],
  percent: ["tableSortLow", "tableSortHigh"],
  duration: ["tableSortLow", "tableSortHigh"],
  rating: ["tableSortLow", "tableSortHigh"],
  date: ["tableSortEarly", "tableSortLate"],
  check: ["tableSortNoFirst", "tableSortYesFirst"],
};

const ticked = (on: boolean) => (on ? "✓" : undefined);

const filtersOf = (head: Headed, options: string[], ways: ColumnWays): Choice[] => {
  const said = (kind: Filtering["kind"], label: string, value = ""): Choice => ({
    key: `${kind}-${value}`,
    label,
    onPick: () =>
      ways.filter({ kind, said: value, label: fill("tableFilterChip", head.name, label) }),
  });
  const type = head.type || "text";
  if (type === "select" || type === "tags")
    return options.map((one) => said("is", one, one.toLowerCase()));
  if (type === "check") return [said("yes", t("tableFilterYes")), said("no", t("tableFilterNo"))];
  if (type === "date")
    return [said("soon", t("tableFilterSoon")), said("past", t("tableFilterPast"))];
  if (MEASURED.has(type)) return [];
  return [{ key: "has", label: t("tableFilterHas"), onPick: ways.asking }];
};

/** A column's menu: what makes sense for its type first, then what every column has. */
export const columnChoices = (head: Headed, options: string[], ways: ColumnWays): Choice[] => {
  const type = head.type || "text";
  const [up, down] = SORTS[type] ?? ["tableSortAz", "tableSortZa"];
  const filters = filtersOf(head, options, ways);
  const totals: Choice[] = TOTALS.filter(([word]) => addsUp(type, word)).map(([word, label]) => ({
    key: word,
    label: t(label),
    icon: ticked(head.total === word),
    onPick: () => ways.retype(wordsOf(type, head.currency, word)),
  }));
  const choices: Choice[] = [
    { key: "up", label: t(up), icon: "↑", onPick: () => ways.sort(false) },
    { key: "down", label: t(down), icon: "↓", onPick: () => ways.sort(true) },
    {
      key: "filter",
      label: t("tableFilter"),
      off: filters.length === 0,
      into: { label: t("tableFilter"), choices: filters },
    },
    {
      key: "total",
      label: t("tableTotal"),
      apart: true,
      into: {
        label: t("tableTotal"),
        choices: [
          ...totals,
          {
            key: "none",
            label: t("tableTotalNone"),
            icon: ticked(!head.total),
            onPick: () => ways.retype(wordsOf(type, head.currency, "")),
          },
        ],
      },
    },
  ];
  if (type === "money") {
    choices.push({
      key: "currency",
      label: t("tableCurrency"),
      into: {
        label: t("tableCurrency"),
        choices: [
          {
            key: "none",
            label: t("tableCurrencyNone"),
            icon: ticked(!head.currency),
            onPick: () => ways.retype(wordsOf("money", "", head.total)),
          },
          ...CURRENCIES.map((code) => ({
            key: code,
            label: code,
            icon: ticked(head.currency === code),
            onPick: () => ways.retype(wordsOf("money", code, head.total)),
          })),
        ],
      },
    });
  }
  return [
    ...choices,
    {
      key: "type",
      label: t("tableType"),
      apart: true,
      into: {
        label: t("tableType"),
        choices: TYPES.map(([word, label]) => ({
          key: word,
          label: t(label),
          icon: ticked(type === word),
          // A new type keeps its total only where that total still adds up.
          onPick: () => ways.retype(wordsOf(word, "", addsUp(word, head.total) ? head.total : "")),
        })),
      },
    },
    { key: "rename", label: t("tableRename"), onPick: ways.rename },
    { key: "left", label: t("tableInsertLeft"), onPick: () => ways.insert(false) },
    { key: "right", label: t("tableInsertRight"), onPick: () => ways.insert(true) },
    { key: "markdown", label: t("tableAsMarkdown"), onPick: ways.markdown },
    {
      key: "remove",
      label: t("tableColumnRemove"),
      icon: "✕",
      danger: true,
      apart: true,
      onPick: ways.remove,
    },
  ];
};
