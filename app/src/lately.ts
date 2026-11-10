import type { Filed, Papers } from "./core";
import { daysFrom } from "./format";
import { shown } from "./tongues";

const RECENT = 3;
const WEEK = 7;

export interface Lately {
  today: number;
  week: number;
  folders: number;
  marked: number;
  recent: Filed[];
}

const writtenAt = (doc: Filed): number => (doc.wrote ? Date.parse(doc.wrote) : Number.NaN);

export function lately(papers: Papers, now = new Date()): Lately {
  const docs = shown(papers.docs).filter((one) => !one.away && !one.gone && !one.pageOf);
  const written = docs
    .filter((one) => !Number.isNaN(writtenAt(one)))
    .sort((a, b) => writtenAt(b) - writtenAt(a));
  const within = (days: number) =>
    written.filter((one) => -daysFrom(one.wrote ?? "", now) < days).length;
  return {
    today: within(1),
    week: within(WEEK),
    folders: papers.folders.filter((one) => !one.away).length,
    marked: docs.filter((one) => one.flagged).length,
    recent: written.slice(0, RECENT),
  };
}
