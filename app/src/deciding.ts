import { ask } from "@tauri-apps/plugin-dialog";
import { docs, paperRifts, settlePaper, weavePaper, type Pick, type Rift } from "./core";
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

export const decideAll = async (ids: string[]): Promise<void> => {
  if (!ids.length) return;
  const titled = await docs()
    .then((found) => new Map(found.docs.map((one) => [one.file, one.title])))
    .catch(() => new Map<string, string>());
  for (const id of ids) await decide(id, titled.get(id));
};
