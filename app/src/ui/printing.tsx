import { attached, docFacts, type Paper } from "../core";
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
  const [{ generateJSON }, { written: shapes, loosened }, { composed }] = await Promise.all([
    import("@tiptap/core"),
    import("./writing"),
    import("../markdown"),
  ]);
  const sheets = await Promise.all(
    [first, ...rest.map((body) => generateJSON(composed(loosened(body)), shapes()))].map((one) =>
      fetched(shapesOf(one), attached, at),
    ),
  );
  return pdf(<Papered sheets={sheets} leaf={leaf} signed={signed} />).toBlob();
};

export const pdfOfText = async (body: string, leaf: Paper, signed?: string): Promise<Blob> => {
  const [{ generateJSON }, { written: shapes, loosened }, { composed }] = await Promise.all([
    import("@tiptap/core"),
    import("./writing"),
    import("../markdown"),
  ]);
  return pdfOf(generateJSON(composed(loosened(body)), shapes()), [], leaf, signed);
};
