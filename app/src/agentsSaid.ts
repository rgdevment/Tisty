import { onlyHooks, type Parts, partsIn, type State } from "./agentsAmiss";
import type { Wired } from "./core";
import { fill, t, type Word } from "./locales";

type Kind = "connection" | "hooks" | "mixed";

const kindOf = (all: readonly Wired[], state: State): Kind => {
  const only = all.filter((one) => onlyHooks(partsIn(one, state))).length;
  if (only === all.length) return "hooks";
  return only === 0 ? "connection" : "mixed";
};

const ONE: Record<State, Record<Exclude<Kind, "mixed">, Word>> = {
  lost: { connection: "attAstray", hooks: "attAstrayHooks" },
  elsewhere: { connection: "attElsewhere", hooks: "attElsewhereHooks" },
};

const MANY: Record<State, Record<Kind, Word>> = {
  lost: {
    connection: "attAstrayMany",
    hooks: "attAstrayHooksMany",
    mixed: "attAstrayMixed",
  },
  elsewhere: {
    connection: "attElsewhereMany",
    hooks: "attElsewhereHooksMany",
    mixed: "attElsewhereMixed",
  },
};

const NOTICE: Record<Kind, Word> = {
  connection: "amissMany",
  hooks: "amissHooksMany",
  mixed: "amissMixed",
};

export const placeNamed = (one: Wired): string =>
  one.place ? `${one.name} (${fill("wslTitle", one.place)})` : one.name;

const single = (all: readonly Wired[], state: State) =>
  onlyHooks(partsIn(all[0], state)) ? "hooks" : "connection";

export const pendingSaid = (all: readonly Wired[], state: State): string =>
  all.length === 1
    ? fill(ONE[state][single(all, state)], placeNamed(all[0]))
    : fill(MANY[state][kindOf(all, state)], String(all.length));

export const noticeSaid = (all: readonly Wired[]): string =>
  all.length === 1
    ? fill(single(all, "lost") === "hooks" ? "amissHooks" : "amissOne", placeNamed(all[0]))
    : fill(NOTICE[kindOf(all, "lost")], String(all.length));

export const partsSaid = (parts: Parts): string => {
  if (parts.hooks.length === 0) return t("mendConnection");
  return t(onlyHooks(parts) ? "mendHooks" : "mendBoth");
};

export const pathsSaid = (one: Wired, parts: Parts): string[] => [
  ...(parts.connection === undefined
    ? []
    : [fill(one.gone ? "wiringPointsGone" : "wiringPoints", parts.connection)]),
  ...(one.hooks?.stray ?? [])
    .filter((it) => parts.hooks.includes(it.said))
    .map((it) => fill(it.gone ? "hooksPointGone" : "hooksPoint", it.said)),
];
