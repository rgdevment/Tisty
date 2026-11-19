import type { Guess, Network } from "../../brainsty";
import { lineDetail } from "./zoom";

export interface Curve {
  from: number;
  to: number;
  strength: number;
}

export interface Shown {
  curve: Curve;
  fade: number;
  bothInSight: boolean;
}

export interface Showing {
  on: boolean;
  picked: number | null;
  zoom: number;
  hidden: readonly boolean[];
}

export interface Alike {
  node: number;
  words: string[];
}

export interface Part {
  text: string;
  word: boolean;
}

const FAINTEST = 0.3;
const BOLDEST = 0.95;

// Shared words and likeness of sense score on scales of their own, so each guess is weighed within its method.
export function curvesOf(guesses: readonly Guess[]): Curve[] {
  const span = new Map<number, [number, number]>();
  for (const [, , score, by] of guesses) {
    const [low, high] = span.get(by) ?? [score, score];
    span.set(by, [Math.min(low, score), Math.max(high, score)]);
  }
  return guesses.map(([from, to, score, by]) => {
    const [low, high] = span.get(by) ?? [score, score];
    return { from, to, strength: high > low ? (score - low) / (high - low) : 1 };
  });
}

export const alphaOf = (strength: number): number =>
  FAINTEST + (BOLDEST - FAINTEST) * Math.min(1, Math.max(0, strength));

export function curvesShown(curves: readonly Curve[], showing: Showing): Shown[] {
  if (!showing.on) return [];
  const { picked, hidden } = showing;
  const near = lineDetail(showing.zoom);
  return curves.flatMap((curve): Shown[] => {
    if (hidden[curve.from] || hidden[curve.to]) return [];
    if (picked !== null)
      return curve.from === picked || curve.to === picked
        ? [{ curve, fade: 1, bothInSight: false }]
        : [];
    return near > 0 ? [{ curve, fade: near, bothInSight: true }] : [];
  });
}

export function alikeOf(network: Network, node: number, hidden: readonly boolean[]): Alike[] {
  const guesses = network.alike ?? [];
  const curves = curvesOf(guesses);
  return guesses
    .flatMap(([from, to, , , shared], n) => {
      if (from !== node && to !== node) return [];
      const other = from === node ? to : from;
      if (hidden[other]) return [];
      return [
        { node: other, strength: curves[n].strength, words: shared.split(" ").filter(Boolean) },
      ];
    })
    .sort((one, other) => other.strength - one.strength)
    .map(({ node: other, words }) => ({ node: other, words }));
}

const SOUNDS_I = /^h?i(?![aeiouáéíóú])/i;

export function wordsSaid(words: readonly string[], and: string, andBeforeI = and): Part[] {
  return words.flatMap((word, n) => {
    const said = { text: word, word: true };
    if (n === 0) return [said];
    if (n < words.length - 1) return [{ text: ", ", word: false }, said];
    return [{ text: ` ${SOUNDS_I.test(word) ? andBeforeI : and} `, word: false }, said];
  });
}
