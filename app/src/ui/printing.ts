import { docFacts, pagePdf } from "../core";
import { wholeStamp } from "../format";
import { valuesIn } from "../front";
import { t } from "../locales";
import { type PaperSheet, webPage } from "./webPage";

export interface Told {
  readonly label: string;
  readonly said: string;
}

export interface Stamp {
  readonly card?: readonly Told[];
  readonly credited?: boolean;
}

const dated = (when: number): string => wholeStamp(new Date(when * 1000));

const declared = (body: string, key: string): string | undefined =>
  valuesIn(body, key)?.join(", ").trim() || undefined;

// What the document says of itself goes before what Tisty saw happen to it.
export const factsCard = async (file: string, body: string): Promise<Told[] | undefined> => {
  const facts = await docFacts(file).catch(() => null);
  const author = declared(body, "author");
  const rows: [Parameters<typeof t>[0], string | null | undefined][] = [
    ["pdfAuthor", author ?? facts?.signedAs],
    // An author the document declares is not the one the log saw edit it.
    ["pdfEditor", author ? null : facts?.editedAs],
    ["pdfVersion", declared(body, "version")],
    ["pdfDate", declared(body, "date")],
    ["pdfMade", facts?.made ? dated(facts.made) : null],
    ["pdfWrote", facts?.wrote ? dated(facts.wrote) : null],
  ];
  const told = rows.flatMap(([label, said]) => (said ? [{ label: t(label), said }] : []));
  return told.length ? told : undefined;
};

export interface Stamping {
  readonly covered: boolean;
  readonly credited: boolean;
}

export const STAMPING: Stamping = { covered: false, credited: false };

// The card goes on a cover of its own, so the pages after it are the sheets seen on screen.
export const stampOf = async (
  file: string,
  body: string,
  { covered, credited }: Stamping,
): Promise<Stamp> => ({ card: covered ? await factsCard(file, body) : undefined, credited });

// The PDF is the web page on paper, laid out by the window's own engine.
export const pdfOf = async (bodies: string[], sheet: PaperSheet): Promise<Blob> => {
  const page = await webPage(bodies, "", sheet);
  const paper = sheet.landscape ? `${sheet.leaf}-landscape` : sheet.leaf;
  return new Blob([new Uint8Array(await pagePdf(page, paper))], { type: "application/pdf" });
};

export interface Seen {
  /** The blocks that open a sheet on screen, by where they sit in the text. */
  readonly turns: string[];
  /** The folded blocks left open on screen, which print open where they are. */
  readonly opened: string[];
}

const placed = (all: Iterable<Element | null>): string[] =>
  Array.from(all).flatMap((one) => {
    const at = (one as HTMLElement | null)?.dataset.at;
    return at ? [at] : [];
  });

// The page as it is drawn on screen: where its sheets turn and which folds are open.
export const seenIn = (page: Document | null | undefined): Seen => ({
  turns: placed(
    Array.from(page?.querySelectorAll(".page > .sheet-turn") ?? []).map(
      (one) => one.nextElementSibling,
    ),
  ),
  opened: placed(page?.querySelectorAll(".page > details[open]") ?? []),
});
