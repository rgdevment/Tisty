import { ask } from "@tauri-apps/plugin-dialog";
import { docs, settlePaper } from "./core";
import { fill, t } from "./locales";

const settling = new Set<string>();

export const decide = async (id: string, called?: string): Promise<void> => {
  if (settling.has(id)) return;
  settling.add(id);
  const said = called?.trim() || t("untitledDoc");
  try {
    if (await ask(fill("bothChanged", said), { kind: "warning" })) {
      await settlePaper(id, "both");
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
