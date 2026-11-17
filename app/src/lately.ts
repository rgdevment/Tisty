import type { Filed, Papers } from "./core";
import { daysFrom } from "./format";
import { shown } from "./tongues";

const WEEK = 7;

export interface Lately {
  today: number;
  week: number;
  folders: number;
  marked: number;
}

const kept = (papers: Papers): Filed[] =>
  shown(papers.docs).filter((one) => !one.away && !one.gone && !one.pageOf);

export function lately(papers: Papers, now = new Date()): Lately {
  const docs = kept(papers);
  const within = (days: number) =>
    docs.filter((one) => {
      if (!one.wrote) return false;
      const ago = -daysFrom(one.wrote, now);
      return ago >= 0 && ago < days;
    }).length;
  return {
    today: within(1),
    week: within(WEEK),
    folders: papers.folders.filter((one) => !one.away).length,
    marked: docs.filter((one) => one.flagged).length,
  };
}

export const wroteAt = (doc: Filed): number => Date.parse(doc.wrote ?? "") || 0;

export const latest = (papers: Papers, most: number): Filed[] =>
  kept(papers)
    .filter((one) => one.wrote)
    .sort((one, two) => wroteAt(two) - wroteAt(one))
    .slice(0, most);
