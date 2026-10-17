export const TOPICS = {
  properties: ["properties", "propiedades"],
  headings: ["headings", "titulos"],
  bold: ["bold-italic-struck-through-and-code", "negrita-cursiva-tachado-y-codigo"],
  underline: ["underline", "subrayado"],
  pens: ["pens", "rotuladores"],
  links: ["links", "enlaces"],
  lists: ["lists", "listas"],
  callouts: ["quotes-and-callouts", "citas-y-avisos"],
  folds: ["collapsible-sections", "secciones-plegables"],
  tags: ["tags", "etiquetas"],
  icons: ["icons", "iconos"],
  emoji: ["emoji", "emojis"],
  docs: ["other-documents-and-pages", "otros-documentos-y-paginas"],
  attachments: ["attachments", "adjuntos"],
  references: ["references", "referencias"],
  footnotes: ["footnotes", "notas-al-pie"],
  code: ["code-blocks", "bloques-de-codigo"],
  drawn: ["diagrams-formulas-and-widgets", "diagramas-formulas-y-widgets"],
  tables: ["tables", "tablas"],
  breaks: ["page-breaks", "saltos-de-pagina"],
} as const;

export type Kind = keyof typeof TOPICS;

export const slugOf = (kind: Kind, spanish: boolean): string => TOPICS[kind][spanish ? 1 : 0];

const DRAWN = ["mermaid", "mmd", "math", "widget"];

const shaped = (block: string): Kind | null => {
  const said = block.replace(/^\s+/, "");
  const fence = /^(`{3,}|~{3,})\s*([\w-]*)/.exec(said);
  if (fence) return DRAWN.includes(fence[2].toLowerCase()) ? "drawn" : "code";
  if (said.startsWith("$$")) return "drawn";
  if (/^---\s*$/.test(said.trim())) return "breaks";
  if (/^---\n[\w-]+\s*:/.test(said)) return "properties";
  if (said.startsWith("|")) return "tables";
  if (said.startsWith(":::details")) return "folds";
  if (/^:::(note|tip|important|warning|caution)\b/i.test(said)) return "callouts";
  if (said.startsWith(">")) return "callouts";
  if (/^<details/i.test(said)) return "folds";
  if (/^#{1,6} /.test(said)) return "headings";
  if (/^\[\^[^\]]+\]:/.test(said)) return "footnotes";
  if (/^([-*+]|\d+[.)]) /.test(said)) return "lists";
  return null;
};

const marked = (line: string): Kind | null => {
  if (/==[^=\s][^=]*==/.test(line)) return "pens";
  if (/\+\+[^+\s][^+]*\+\+/.test(line)) return "underline";
  if (/::[a-z0-9-]+( [a-z]+)?::/.test(line)) return "icons";
  if (/\[\[[^\]]+\]\]/.test(line)) return "references";
  if (/tisty:doc\//.test(line)) return "docs";
  if (/!\[[^\]]*\]\(<?[^)\s/]+-[0-9a-f]{8,16}\.\w+>?\)/.test(line)) return "attachments";
  if (/\[\^[^\]]+\]/.test(line)) return "footnotes";
  if (/\]\(|https?:\/\/|<[^>\s]+@[^>\s]+>/.test(line)) return "links";
  if (/\$[^$\s][^$]*\$/.test(line)) return "drawn";
  if (/(^|\s)#\p{L}[\p{L}\p{N}_-]*/u.test(line)) return "tags";
  if (/(^|\s):[a-z][a-z0-9_+-]*:(?=$|[\s.,;!?])/.test(line)) return "emoji";
  if (/\*\*|~~|`|(^|\s)\*[^*\s]/.test(line)) return "bold";
  return null;
};

// What a block is shaped as says the most; otherwise what the line under the caret carries.
export const kindOf = (block: string, line: string = block): Kind | null =>
  shaped(block) ?? marked(line);
