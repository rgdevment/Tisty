import type { Say } from "tisty-sheet";
import type { Act, Facts, More } from "./side";
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
      fronted(page, say, facts),
      tagged(page, say, facts),
    ];
    into.append(...parts.filter((one): one is HTMLElement => one !== null));
  });
};

const acted = (page: Document, act: Act) =>
  pressed(page, `side-act${act.main ? " is-main" : ""}`, act.label, (at) => act.onPick?.(at));

export const besideMore = (page: Document, say: Say, panel: More): HTMLElement => {
  switch (panel.kind) {
    case "details":
      return detailed(page, say, panel);
    case "acts": {
      const acts = made(page, "div", "side-acts");
      for (const act of panel.acts) acts.append(acted(page, act));
      return acts;
    }
  }
};
