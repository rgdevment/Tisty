import { message } from "@tauri-apps/plugin-dialog";
import { attach } from "./core";
import { fill, t } from "./locales";

// Photos a window on another computer may not draw; the rest reads the same everywhere.
const NOT_EVERYWHERE = ["heic", "heif", "tif", "tiff", "bmp"];
// Only Windows and macOS carry what turns them; elsewhere they are kept as they came.
const CAN_TURN = ["Macintosh", "Windows"].some((one) => navigator.userAgent.includes(one));

const leaf = (path: string): string => path.split(/[\\/]/).pop() ?? path;
const ending = (path: string): string => leaf(path).split(".").pop()?.toLowerCase() ?? "";

export const turnable = (path: string): boolean =>
  CAN_TURN && NOT_EVERYWHERE.includes(ending(path));

/** Asked once for every photo dropped together, then each one is attached the way that was chosen. */
export async function attachedAll(paths: string[]): Promise<Promise<string>[]> {
  const asked = paths.filter(turnable);
  if (asked.length === 0) return paths.map((one) => attach(one));
  const full = t("reshapeFull");
  const light = t("reshapeLight");
  const said =
    asked.length === 1
      ? fill("reshapeAskOne", leaf(asked[0]))
      : fill("reshapeAskMany", String(asked.length));
  const chosen = await message(said, {
    title: t("reshapeTitle"),
    kind: "info",
    buttons: { yes: full, no: light, cancel: t("reshapeKeep") },
  });
  const shape = chosen === full ? "full" : chosen === light ? "light" : undefined;
  return paths.map((one) => attach(one, undefined, turnable(one) ? shape : undefined));
}
