import type { Papers } from "./core";
import { folded } from "./folding";

export function sifted(papers: Papers, query: string): Papers {
  const asked = folded(query.trim());
  if (!asked) return papers;
  const says = (text: string) => folded(text).includes(asked);
  const parentOf = new Map(papers.folders.map((one) => [one.id, one.parent]));
  const kidsOf = new Map<string, string[]>();
  for (const one of papers.folders) {
    if (one.parent) kidsOf.set(one.parent, [...(kidsOf.get(one.parent) ?? []), one.id]);
  }

  const whole = new Set<string>();
  const sink = (id: string) => {
    if (whole.has(id)) return;
    whole.add(id);
    for (const kid of kidsOf.get(id) ?? []) sink(kid);
  };
  for (const folder of papers.folders) if (says(folder.name)) sink(folder.id);

  const docs = new Set<string>();
  for (const doc of papers.docs) {
    if (!says(doc.title) && !(doc.folder && whole.has(doc.folder))) continue;
    docs.add(doc.id);
    if (doc.pageOf) docs.add(doc.pageOf);
  }

  const folders = new Set<string>();
  const climb = (from: string | null) => {
    for (let at = from; at && !folders.has(at); at = parentOf.get(at) ?? null) folders.add(at);
  };
  for (const id of whole) climb(id);
  for (const doc of papers.docs) if (docs.has(doc.id)) climb(doc.folder);

  return {
    folders: papers.folders.filter((one) => folders.has(one.id)),
    docs: papers.docs.filter((one) => docs.has(one.id)),
  };
}
