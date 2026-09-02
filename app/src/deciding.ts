import { ask } from "@tauri-apps/plugin-dialog";
import { docs, type Pick, paperRifts, type Rift, settlePaper, weavePaper } from "./core";
import { fill, t } from "./locales";

const settling = new Set<string>();

type Asking = (named: string, rifts: Rift[]) => Promise<Pick[] | null>;

let byBlock: Asking | null = null;

export const decidesByBlock = (asking: Asking | null): void => {
  byBlock = asking;
};

export const decide = async (id: string, called?: string): Promise<void> => {
  if (settling.has(id)) return;
  settling.add(id);
  const said = called?.trim() || t("untitledDoc");
  try {
    if (byBlock) {
      const torn = await paperRifts(id).catch(() => null);
      if (torn?.rifts.length) {
        const picks = await byBlock(said, torn.rifts);
        if (!picks) return;
        const wove = await weavePaper(id, picks, torn.print).then(
          () => true,
          () => false,
        );
        if (wove) {
          await settlePaper(id, "mine");
          return;
        }
      }
    }
    if (await ask(fill("bothChanged", said), { kind: "warning" })) {
      await settlePaper(id, "both", t("otherVersion"));
      return;
    }
    const mine = await ask(fill("whoseWins", said), { kind: "warning" });
    await settlePaper(id, mine ? "mine" : "theirs");
  } finally {
    settling.delete(id);
  }
};

export const decideAll = async (ids: string[]): Promise<string[]> => {
  if (!ids.length) return [];
  const found = await docs().catch(() => null);
  if (!found) return ids;
  const titled = new Map(found.docs.map((one) => [one.file, one.title]));
  const shut = new Set(found.docs.filter((one) => one.locked).map((one) => one.file));
  for (const id of ids) {
    if (shut.has(id)) continue;
    try {
      await decide(id, titled.get(id));
    } catch {
      shut.add(id);
    }
  }
  return ids.filter((id) => shut.has(id));
};
