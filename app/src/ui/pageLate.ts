import { applied, changeOf, type Edit, linedAs, type Span } from "./pageEdits";
import type { Writer } from "./pageWriter";

export type Laid = { readonly edit: Edit; readonly caret: number };

export type Asked = (now: string) => Span | null;

export type Late = (
  asked: Asked,
  laid: (now: string, at: Span) => Laid,
  after?: (one: Laid) => void,
) => boolean;

const pointIn = (moved: Edit, at: number, onward: boolean): number | null => {
  const by = moved.insert.length - (moved.to - moved.from);
  if (moved.to < at || (moved.to === at && (moved.from < at || onward))) return at + by;
  return moved.from >= at ? at : null;
};

const spanCarried = (text: string, now: string, from: number, to: number, stretches: boolean) => {
  if (now === text) return { from, to };
  const moved = changeOf(text, now);
  if (!stretches && moved.from < to && moved.to > from) return null;
  const start = pointIn(moved, from, !stretches && from < to);
  const end = pointIn(moved, to, stretches);
  return start === null || end === null ? null : { from: start, to: end };
};

export const wordsAsked =
  (text: string, one: Span): Asked =>
  (now) =>
    spanCarried(text, now, one.from, one.to, false);

export const blockAsked =
  (text: string, from: number, end: number): Asked =>
  (now) =>
    spanCarried(text, now, from, end, true);

const alone = (now: string, one: Laid) => applied(now, linedAs(now, one.edit, one.caret)[0]);

export const lateOn = (
  writer: Writer,
  put: (wanted: Edit, at: number) => void,
  stopped: () => boolean,
): Late => {
  const { hands } = writer;
  const seat = hands.seat?.();
  return (asked, laid, after) => {
    const landed = (now: string) => {
      const at = asked(now);
      return at && laid(now, at);
    };
    if (hands.seat?.() === seat && !stopped()) {
      const one = landed(hands.text());
      if (!one) return false;
      put(one.edit, one.caret);
      after?.(one);
      return true;
    }
    return (
      seat?.current?.((now) => {
        const one = landed(now);
        return one && alone(now, one);
      }) ?? false
    );
  };
};
