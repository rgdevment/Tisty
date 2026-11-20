import { type Card, type Neuron, TIES, type Tie } from "../../brainsty";
import { dayOf, numbered, weigh } from "../../format";
import { fill, t } from "../../locales";
import { standingOf } from "./looks";

export type Fact = [label: string, value: string];

export const CARDED = new Set(["doc", "page", "task"]);

// A task's day has no hour, and read as midnight in UTC it would fall on the day before here.
const onDay = (iso: string): string => dayOf(`${iso}T00:00:00`);

const sizeOf = (card: Card): string | null => {
  if (card.bytes === undefined) return null;
  return card.words
    ? fill("brainstyFactWords", weigh(card.bytes), numbered(card.words))
    : weigh(card.bytes);
};

const stateOf = (neuron: Neuron, card: Card): string => {
  const state = t(`brainstyStanding_${standingOf(neuron)}`);
  const due = card.standing;
  return due && due !== "undated" ? `${state} · ${t(`brainstyDue_${due}`)}` : state;
};

export function factsOf(neuron: Neuron, card: Card): Fact[] {
  const size = sizeOf(card);
  const facts: (Fact | null)[] =
    neuron.kind === "task"
      ? [
          [t("brainstyFactState"), stateOf(neuron, card)],
          card.date ? [t("brainstyFactDate"), onDay(card.date)] : null,
          card.deadline ? [t("brainstyFactDeadline"), onDay(card.deadline)] : null,
          card.part_of ? [t("brainstyFactPartOf"), card.part_of.title] : null,
          card.parts?.length ? [t("brainstyFactParts"), numbered(card.parts.length)] : null,
          card.steps
            ? [
                t("brainstyFactSteps"),
                fill("brainstyFactStepsDone", numbered(card.steps[0]), numbered(card.steps[1])),
              ]
            : null,
          card.source ? [t("brainstyFactSource"), card.source] : null,
        ]
      : [
          card.book ? [t("brainstyFactBook"), card.book.title] : null,
          card.wrote ? [t("brainstyFactWrote"), dayOf(card.wrote)] : null,
          card.made ? [t("brainstyFactMade"), dayOf(card.made)] : null,
          size ? [t("brainstyFactSize"), size] : null,
          card.lang ? [t("brainstyFactLanguage"), card.lang.toUpperCase()] : null,
        ];
  return facts.filter((one): one is Fact => one !== null);
}

const USES = TIES.indexOf("uses");

export function usersOf(
  network: { nodes: readonly Neuron[]; edges: readonly (readonly number[])[] },
  file: number,
  hidden: readonly boolean[],
): Neuron[] {
  const users = network.edges.flatMap(([from, to, code]) =>
    code === USES && to === file && !hidden[from] ? [network.nodes[from]] : [],
  );
  return users.sort((one, other) => one.title.localeCompare(other.title));
}

export function tiesAt(edges: readonly (readonly number[])[], node: number): [Tie, number][] {
  const counts = new Map<Tie, number>();
  for (const [from, to, code] of edges) {
    const tie = TIES[code];
    if (tie && (from === node || to === node)) counts.set(tie, (counts.get(tie) ?? 0) + 1);
  }
  return TIES.flatMap((tie) => (counts.has(tie) ? [[tie, counts.get(tie) ?? 0]] : []));
}
