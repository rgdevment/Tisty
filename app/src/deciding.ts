import { ask } from "@tauri-apps/plugin-dialog";
import { docs, type Pick, paperRifts, type Rift, settlePaper, weavePaper } from "./core";
import { fill, t } from "./locales";
import { type Refusal, saidPlainly } from "./refusal";

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

/** What deciding left: the documents locked here, and the plain reason anything else failed. */
export interface Decided {
  shut: string[];
  said: string | null;
}

export const decideAll = async (ids: string[]): Promise<Decided> => {
  if (!ids.length) return { shut: [], said: null };
  let said: string | null = null;
  const found = await docs().catch((problem: unknown) => {
    said = saidPlainly(problem);
    return null;
  });
  if (!found) return { shut: [], said };
  const titled = new Map(found.docs.map((one) => [one.file, one.title]));
  const shut = new Set(found.docs.filter((one) => one.locked).map((one) => one.file));
  for (const id of ids) {
    if (shut.has(id)) continue;
    try {
      await decide(id, titled.get(id));
    } catch (problem) {
      const code = (problem as Refusal | undefined)?.code;
      if (code === "documentLocked") {
        shut.add(id);
        continue;
      }
      said = saidPlainly(problem);
      if (code === "movedUnderfoot") {
        said = fill("changedWhileDeciding", titled.get(id)?.trim() || t("untitledDoc"));
      }
    }
  }
  return { shut: ids.filter((id) => shut.has(id)), said };
};
