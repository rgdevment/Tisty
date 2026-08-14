import { ask } from "@tauri-apps/plugin-dialog";
import { settlePaper } from "./core";
import { fill } from "./locales";

export const decide = async (id: string): Promise<void> => {
  if (await ask(fill("bothChanged", id), { kind: "warning" })) {
    await settlePaper(id, "both");
    return;
  }
  const mine = await ask(fill("whoseWins", id), { kind: "warning" });
  await settlePaper(id, mine ? "mine" : "theirs");
};

export const decideAll = async (ids: string[]): Promise<void> => {
  for (const id of ids) await decide(id);
};
