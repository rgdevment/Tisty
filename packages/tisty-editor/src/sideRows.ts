import type { Say } from "tisty-sheet";
import type { Act, Facts, Held, More, Prints } from "./side";
import { wordsIn } from "./side";
import { listed, made, pressed, rowed } from "./sideParts";

type Of<K extends More["kind"]> = Extract<More, { kind: K }>;

// The title and the tags are shown above and beside the details, so the properties leave them out.
const SHOWN = new Set(["title", "tags", "tag", "lang", "translationkey"]);

const fronted = (page: Document, say: Say, facts: Facts) => {
  const fields = (facts.fields ?? []).filter((one) => !SHOWN.has(one.key.toLowerCase()));
  if (!fields.length && !facts.overgrown && !facts.onFront) return null;
  const part = made(page, "section", "side-part");
  const head = made(page, "div", "side-part-head");
  head.append(made(page, "h4", "side-label", say("frontNamed")));
  const { onFront } = facts;
  if (onFront) head.append(pressed(page, "side-link", say("frontEdit"), () => onFront()));
  part.append(head);
  if (facts.overgrown) part.append(made(page, "p", "side-none", say("frontOvergrown")));
  if (fields.length)
    part.append(
      listed(
        page,
        fields.map((one) => [one.key, one.values.join(", ")] as const),
      ),
    );
  return part;
};

const tagged = (page: Document, say: Say, facts: Facts) => {
  const tags = facts.tags ?? [];
  if (!tags.length) return null;
  const part = made(page, "section", "side-part");
  part.append(made(page, "h4", "side-label", say("tags")));
  const chips = made(page, "div", "side-tags");
  for (const one of tags) {
    const chip = made(page, "span", "side-tag", `#${one.name}`);
    if (one.times > 0) {
      chip.title = say("tagTimes", `${one.times}`);
      chip.append(made(page, "span", "side-times", `${one.times}`));
    }
    chips.append(chip);
  }
  part.append(chips);
  return part;
};

const detailed = (page: Document, say: Say, panel: Of<"details">) => {
  const { facts } = panel;
  const words = facts.body === undefined ? undefined : wordsIn(facts.body);
  const long =
    words === undefined ? undefined : words ? say("paperWords", `${words}`) : say("paperEmpty");
  const gist = [long, facts.weighs].filter(Boolean).join(" · ");
  const line = { key: "details", label: say("besideDetails"), gist, open: panel.open };
  return rowed(page, line, (into) => {
    const parts = [
      listed(page, [
        [say("paperAuthor"), facts.author],
        [say("paperBorn"), facts.born],
        [say("paperEditor"), facts.editor],
        [say("paperMade"), facts.made],
        [say("paperWrote"), facts.wrote],
        [say("paperLong"), long],
        [say("paperWeighs"), facts.weighs],
        [say("paperHolds"), facts.holds],
      ]),
      facts.edited ? printed(page, facts.edited) : null,
      fronted(page, say, facts),
      tagged(page, say, facts),
    ];
    into.append(...parts.filter((one): one is HTMLElement => one !== null));
  });
};

const acted = (page: Document, act: Act) => {
  const kinds = `${act.main ? " is-main" : ""}${act.danger ? " is-danger" : ""}`;
  const one = pressed(page, `side-act${kinds}`, act.label, (at) => act.onPick?.(at));
  one.disabled = Boolean(act.off);
  return one;
};

const linked = (page: Document, held: Held) => {
  const link = made(page, "div", "side-held");
  const head = made(page, "div", "side-part-head");
  head.append(made(page, "span", "side-held-label", held.label));
  head.append(pressed(page, "side-link", held.copy, () => held.onCopy()));
  link.append(head, made(page, "span", "side-held-url", held.url));
  if (held.note) link.append(made(page, "span", "side-held-note", held.note));
  return link;
};

const printed = (page: Document, prints: Prints) => {
  const part = made(page, "section", "side-part side-prints");
  part.append(
    made(page, "h4", "side-label", prints.label),
    listed(page, prints.rows),
    made(page, "p", "side-held-note", prints.note),
  );
  return part;
};

const heldRow = (page: Document, panel: Of<"held">) => {
  const line = { key: panel.key, label: panel.label, gist: panel.gist, open: panel.open };
  const row = rowed(page, line, (into) => {
    for (const said of panel.said ?? []) into.append(made(page, "p", "side-said", said));
    if (panel.prints?.rows.length) into.append(printed(page, panel.prints));
    for (const one of panel.links ?? []) into.append(linked(page, one));
    if (panel.acts?.length) {
      const acts = made(page, "div", "side-acts is-inside");
      for (const act of panel.acts) acts.append(acted(page, act));
      into.append(acts);
    }
  });
  if (panel.tone) row.querySelector(".side-gist")?.classList.add(`is-${panel.tone}`);
  if (panel.mark)
    row.querySelector(".side-row-label")?.after(made(page, "span", "side-row-mark", panel.mark));
  return row;
};

export const besideMore = (page: Document, say: Say, panel: More): HTMLElement => {
  switch (panel.kind) {
    case "details":
      return detailed(page, say, panel);
    case "acts": {
      const acts = made(page, "div", "side-acts");
      for (const act of panel.acts) acts.append(acted(page, act));
      return acts;
    }
    case "held":
      return heldRow(page, panel);
  }
};
