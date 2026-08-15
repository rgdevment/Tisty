import { ask } from "@tauri-apps/plugin-dialog";
import { settlePaper } from "./core";
import { fill } from "./locales";

const settling = new Set<string>();

export const decide = async (id: string): Promise<void> => {
  if (settling.has(id)) return;
  settling.add(id);
  try {
    if (await ask(fill("bothChanged", id), { kind: "warning" })) {
      await settlePaper(id, "both");
      return;
    }
    const mine = await ask(fill("whoseWins", id), { kind: "warning" });
    await settlePaper(id, mine ? "mine" : "theirs");
  } finally {
    settling.delete(id);
  }
};

export const decideAll = async (ids: string[]): Promise<void> => {
  for (const id of ids) await decide(id);
};
