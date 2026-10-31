import type { Act, Facts, Link, More } from "./side";
import { wordsIn } from "./side";
import { listed, made, pressed, rowed } from "./sideParts";
import type { Say } from "./words";

type Of<K extends More["kind"]> = Extract<More, { kind: K }>;

// The title and the tags are shown above and beside the details, so the properties leave them out.
const SHOWN = new Set(["title", "tags", "tag"]);

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
  const words = wordsIn(facts.body);
  const long = words ? say("paperWords", `${words}`) : say("paperEmpty");
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

const VERDICT = {
  holds: ["signedHolds", "signedHoldsSaid", "good"],
  revoked: ["signedRevoked", "signedRevokedSaid", "warn"],
  refused: ["signedRefused", "signedRefusedSaid", "bad"],
  unreadable: ["signedUnreadable", "signedUnreadableSaid", "bad"],
} as const;

// Folded until asked: whose key, its print and what it says; a body nobody signed has nothing more to show.
const sealed = (page: Document, say: Say, { seal }: Of<"seal">) => {
  if (seal.verdict === "unsigned")
    return rowed(page, { key: "seal", label: say("signedTitle"), gist: say("signedNone") });
  const [short, long, tone] = VERDICT[seal.verdict];
  const gist = [seal.name, seal.who, say(short)].filter(Boolean).join(" · ");
  return rowed(page, { key: "seal", label: say("signedTitle"), gist, tone }, (into) => {
    into.append(made(page, "p", "side-said", say(long)));
    if (seal.signedOn)
      into.append(made(page, "p", "side-said", `${say("signedOn")} ${seal.signedOn}`));
    if (seal.fingerprint) into.append(made(page, "code", "side-print", seal.fingerprint));
    if (seal.sender)
      into.append(made(page, "p", "side-said", `${say("signedSentBy")} ${seal.sender}`));
  });
};

const COPIED_MS = 2000;

// The link is copied as it is shown, and the press says so for a moment.
const copied = (page: Document, say: Say, link: Link) => {
  const part = made(page, "div", "side-link-row");
  const label = made(page, "span", "side-link-label", link.label);
  if (link.note) label.append(made(page, "span", "side-link-note", ` · ${link.note}`));
  const url = made(page, "code", "side-print side-link-url", link.url);
  const copy = pressed(page, "side-act", say("viewCopy"), (at) => {
    page.defaultView?.navigator.clipboard.writeText(link.url).then(
      () => {
        at.textContent = say("viewCopied");
        page.defaultView?.setTimeout(() => {
          at.textContent = say("viewCopy");
        }, COPIED_MS);
      },
      () => undefined,
    );
  });
  const line = made(page, "div", "side-link-line");
  line.append(url, copy);
  part.append(label, line);
  return part;
};

const shared = (page: Document, say: Say, panel: Of<"shared">) => {
  const { said, links = [], acts = [] } = panel;
  const line = {
    key: "shared",
    label: say("besideShared"),
    gist: panel.gist,
    tone: panel.tone,
    open: panel.open,
  };
  const fill =
    said || links.length || acts.length
      ? (into: HTMLElement) => {
          if (said) into.append(made(page, "p", "side-said", said));
          for (const link of links) into.append(copied(page, say, link));
          if (acts.length) {
            const row = made(page, "div", "side-acts is-inside");
            for (const act of acts) row.append(acted(page, act));
            into.append(row);
          }
        }
      : undefined;
  return rowed(page, line, fill);
};

const acted = (page: Document, act: Act) => {
  const className = `side-act${act.main ? " is-main" : ""}`;
  if (!act.href) return pressed(page, className, act.label, (at) => act.onPick?.(at));
  const link = made(page, "a", className, act.label);
  link.href = act.href;
  if (act.download) link.download = act.download;
  if (act.onPick) link.addEventListener("click", () => act.onPick?.(link));
  return link;
};

export const besideMore = (page: Document, say: Say, panel: More): HTMLElement => {
  switch (panel.kind) {
    case "details":
      return detailed(page, say, panel);
    case "seal":
      return sealed(page, say, panel);
    case "shared":
      return shared(page, say, panel);
    case "acts": {
      const acts = made(page, "div", "side-acts");
      for (const act of panel.acts) acts.append(acted(page, act));
      return acts;
    }
  }
};
