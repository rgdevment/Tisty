import { open, save } from "@tauri-apps/plugin-dialog";
import {
  chooseSync,
  joinThem,
  keepSettings,
  mergeStores,
  type Settings,
  syncNow,
  takeOver,
  tidyMerged,
} from "./core";
import { settled } from "./knocking";
import type { Door } from "./ui/Apart";

const NAMED: Record<Door, string> = {
  merge: "tisty-before-joining-both",
  mine: "tisty-folder-before",
  theirs: "tisty-before-joining",
};

export const stillApart = (problem: unknown): boolean => {
  const refusal = problem as { code?: string } | undefined;
  return (
    refusal?.code === "wouldReset" ||
    refusal?.code === "otherStore" ||
    refusal?.code === "restoredApart"
  );
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
  if (door === "merge") {
    await mergeStores(at);
    // Only once the other side has come in is anything there twice.
    await syncNow("pull").catch(() => undefined);
    await tidyMerged().catch(() => undefined);
  } else if (door === "mine") await takeOver(at);
  else await joinThem(at);
  return true;
};

// Large attachments kept only in the folder come home first, so turning it off leaves nothing behind.
export const turnedOff = (kept: Settings | null): Promise<void> =>
  chooseSync(undefined)
    .catch((problem) => {
      if (!kept || (problem as { code?: string } | undefined)?.code !== "sharedAwayToLeave")
        throw problem;
      return keepSettings({ ...kept, holds: "everywhere" })
        .then(() => syncNow("pull"))
        .then((said) => {
          if (said.carried === "busy") throw problem;
          return chooseSync(undefined);
        })
        .catch((again) => keepSettings(kept).then(() => Promise.reject(again)));
    })
    .then(settled);
