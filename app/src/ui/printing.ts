import { docFacts, type Paper, pagePdf } from "../core";
import { wholeStamp } from "../format";
import { valuesIn } from "../front";
import { t } from "../locales";
import { webPage } from "./webPage";

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
export const cardOf = async (file: string, body: string): Promise<Told[] | undefined> => {
  const facts = await docFacts(file).catch(() => null);
  const rows: [Parameters<typeof t>[0], string | null | undefined][] = [
    ["pdfAuthor", declared(body, "author") ?? facts?.signedAs],
    ["pdfEditor", facts?.editedAs],
    ["pdfVersion", declared(body, "version")],
    ["pdfDate", declared(body, "date")],
    ["pdfMade", facts?.made ? dated(facts.made) : null],
    ["pdfWrote", facts?.wrote ? dated(facts.wrote) : null],
  ];
  const told = rows.flatMap(([label, said]) => (said ? [{ label: t(label), said }] : []));
  return told.length ? told : undefined;
};

export interface Stamping {
  readonly carded: boolean;
  readonly credited: boolean;
}

export const STAMPING: Stamping = { carded: true, credited: false };

export const stampOf = async (
  file: string,
  body: string,
  { carded, credited }: Stamping,
): Promise<Stamp> => ({ card: carded ? await cardOf(file, body) : undefined, credited });

// The PDF is the web page on paper, laid out by the window's own engine.
export const pdfOf = async (
  bodies: string[],
  leaf: Paper,
  stamp?: Stamp,
  landscape = false,
): Promise<Blob> => {
  const page = await webPage(bodies, "", { leaf, landscape, stamp });
  const paper = landscape ? `${leaf}-landscape` : leaf;
  return new Blob([new Uint8Array(await pagePdf(page, paper))], { type: "application/pdf" });
};
