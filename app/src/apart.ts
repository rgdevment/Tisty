import { open, save } from "@tauri-apps/plugin-dialog";
import { chooseSync, joinThem, mergeStores, takeOver } from "./core";
import type { Door } from "./ui/Apart";

const NAMED: Record<Door, string> = {
  merge: "tisty-before-joining-both",
  mine: "tisty-folder-before",
  theirs: "tisty-before-joining",
};

export const stillApart = (problem: unknown): boolean => {
  const refusal = problem as { code?: string } | undefined;
  return refusal?.code === "wouldReset" || refusal?.code === "otherStore";
};

export const walkThrough = async (door: Door | "else"): Promise<boolean> => {
  if (door === "else") {
    const where = await open({ directory: true });
    if (typeof where !== "string") return false;
    await chooseSync(where);
    return true;
  }
  const day = new Date().toISOString().slice(0, 10);
  const at = await save({
    defaultPath: `${NAMED[door]}-${day}.zip`,
    filters: [{ name: "Tisty", extensions: ["zip"] }],
  });
  if (typeof at !== "string") return false;
  if (door === "merge") await mergeStores(at);
  else if (door === "mine") await takeOver(at);
  else await joinThem(at);
  return true;
};
