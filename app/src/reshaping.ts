import { message } from "@tauri-apps/plugin-dialog";
import { attach } from "./core";
import { fill, t } from "./locales";

export type Shape = "full" | "light";
type Asking = (said: string) => Promise<Shape | undefined>;

// Photos a window on another computer may not draw; the rest reads the same everywhere.
const NOT_EVERYWHERE = ["heic", "heif", "tif", "tiff", "bmp"];
// Only Windows and macOS carry what turns them; elsewhere they are kept as they came.
const CAN_TURN = ["Macintosh", "Windows"].some((one) => navigator.userAgent.includes(one));

let asking: Asking | null = null;

/** The window asks in its own dress; the system's dialog is only for when no window is there to. */
export const asksShape = (one: Asking | null): void => {
  asking = one;
};

const leaf = (path: string): string => path.split(/[\\/]/).pop() ?? path;
const ending = (path: string): string => leaf(path).split(".").pop()?.toLowerCase() ?? "";

export const turnable = (path: string): boolean =>
  CAN_TURN && NOT_EVERYWHERE.includes(ending(path));

const askedPlainly: Asking = async (said) => {
  const full = t("reshapeFull");
  const light = t("reshapeLight");
  const chosen = await message(said, {
    title: t("reshapeTitle"),
    kind: "info",
    buttons: { yes: full, no: light, cancel: t("reshapeKeep") },
  });
  return chosen === full ? "full" : chosen === light ? "light" : undefined;
};

/** Asked once for every photo dropped together, then each one is attached the way that was chosen. */
export async function attachedAll(paths: string[]): Promise<Promise<string>[]> {
  const asked = paths.filter(turnable);
  if (asked.length === 0) return inTurn(paths, () => undefined);
  const said =
    asked.length === 1
      ? fill("reshapeAskOne", leaf(asked[0]))
      : fill("reshapeAskMany", String(asked.length));
  const shape = await (asking ?? askedPlainly)(said);
  return inTurn(paths, (one) => (turnable(one) ? shape : undefined));
}

// One after another, as the window kept them before: keeping two at once can store one file twice.
function inTurn(paths: string[], shapeOf: (path: string) => Shape | undefined): Promise<string>[] {
  let before: Promise<unknown> = Promise.resolve();
  return paths.map((one) => {
    const kept = before.then(() => attach(one, undefined, shapeOf(one)));
    before = kept.catch(() => undefined);
    return kept;
  });
}
