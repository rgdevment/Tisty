import { docNew, docs, docWrite, type Folded, folderAdd } from "./core";
import { trail } from "./folders";
import { t } from "./locales";
import { docCard } from "./markdown";
import { fenced, type Start, startOf } from "./ui/drawing/starting";

export interface Born {
  id: string;
  said: string;
}

export const spawned = (name: string, folder?: string, pageOf?: string): Promise<Born> => {
  const said = name.trim();
  if (!said) return Promise.reject(new Error("untitled"));
  return docNew(folder, pageOf)
    .then((made) => docWrite(made.id, `# ${said}\n\n`).then(() => made))
    .then((made) => ({ id: made.id, said: docCard(made.id, said) }));
};

export interface Drafted {
  readonly id: string;
  readonly title: string;
  readonly folder: string;
  // In bytes, as the core counts, since the diagram window finds its block by it.
  readonly at: number;
  readonly source: string;
}

// Diagrams made outside any folder gather in one, found by its name rather than a setting kept somewhere.
const homeOf = async (into?: string): Promise<string> => {
  if (into) return into;
  const named = t("diagramsFolder");
  // The list on screen may be a round behind, and a second folder of the same name is never refused.
  const { folders } = await docs();
  const found = folders.find((one) => !one.away && !one.parent && one.name === named);
  return found ? found.id : folderAdd(named);
};

export const drafted = async (name: string, kind: Start, into?: string): Promise<Drafted> => {
  const said = name.trim();
  if (!said) throw new Error("untitled");
  const folder = await homeOf(into);
  const source = startOf(kind);
  const head = `# ${said}\n\n`;
  const made = await docNew(folder);
  await docWrite(made.id, `${head}${fenced(source)}\n`);
  return { id: made.id, title: said, folder, at: new TextEncoder().encode(head).length, source };
};

// The window names the folder it was made in; one made a moment ago is not on the list yet.
export const placeOf = (folders: Folded[], folder: string): string =>
  trail(folders, folder) || t("diagramsFolder");
