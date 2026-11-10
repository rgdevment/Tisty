import type { Papers } from "./core";

const plain = (text: string): string => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

export function sifted(papers: Papers, query: string): Papers {
  const asked = plain(query.trim());
  if (!asked) return papers;
  const says = (text: string) => plain(text).includes(asked);
  const parentOf = new Map(papers.folders.map((one) => [one.id, one.parent]));
  const folders = new Set<string>();
  const climb = (from: string | null) => {
    for (let at = from; at && !folders.has(at); at = parentOf.get(at) ?? null) folders.add(at);
  };

  const docs = new Set<string>();
  for (const doc of papers.docs) {
    if (!says(doc.title)) continue;
    docs.add(doc.id);
    if (doc.pageOf) docs.add(doc.pageOf);
  }
  for (const doc of papers.docs) if (docs.has(doc.id)) climb(doc.folder);
  for (const folder of papers.folders) if (says(folder.name)) climb(folder.id);

  return {
    folders: papers.folders.filter((one) => folders.has(one.id)),
    docs: papers.docs.filter((one) => docs.has(one.id)),
  };
}
