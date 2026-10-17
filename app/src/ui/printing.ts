import { docFacts, type Paper, pagePdf } from "../core";
import { stamped } from "../format";
import { fill } from "../locales";
import { webPage } from "./webPage";

const dated = (when: number): string => stamped(new Date(when * 1000).toISOString());

export const signature = async (file: string): Promise<string | undefined> => {
  const facts = await docFacts(file).catch(() => null);
  if (!facts) return undefined;
  const said = [
    facts.author ? fill("pdfBy", facts.author) : "",
    facts.editor ? fill("pdfEditedBy", facts.editor) : "",
    facts.made ? fill("pdfMade", dated(facts.made)) : "",
    facts.wrote ? fill("pdfWrote", dated(facts.wrote)) : "",
  ].filter(Boolean);
  return said.length ? said.join("  ·  ") : undefined;
};

// The PDF is the web page on paper, laid out by the window's own engine.
export const pdfOf = async (
  bodies: string[],
  leaf: Paper,
  signed?: string,
  landscape = false,
): Promise<Blob> => {
  const page = await webPage(bodies, "", { leaf, landscape, signed });
  const paper = landscape ? `${leaf}-landscape` : leaf;
  return new Blob([new Uint8Array(await pagePdf(page, paper))], { type: "application/pdf" });
};
