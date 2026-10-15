import { attached, docFacts, docPrinted, type Paper } from "../core";
import { stamped } from "../format";
import { fill } from "../locales";

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

export const pdfOf = async (
  first: unknown,
  rest: string[],
  leaf: Paper,
  signed?: string,
  at?: (file: string) => number | null,
): Promise<Blob> => {
  const [{ pdf }, { Papered, registered }, { fetched, shapesOf }] = await Promise.all([
    import("@react-pdf/renderer"),
    import("./paper"),
    import("./shaping"),
  ]);
  registered();
  const sheets = await Promise.all(
    [first, ...rest].map(async (one) =>
      fetched(shapesOf(typeof one === "string" ? await docPrinted(one) : one), attached, at),
    ),
  );
  return pdf(<Papered sheets={sheets} leaf={leaf} signed={signed} />).toBlob();
};

export const pdfOfText = (body: string, leaf: Paper, signed?: string): Promise<Blob> =>
  pdfOf(body, [], leaf, signed);
