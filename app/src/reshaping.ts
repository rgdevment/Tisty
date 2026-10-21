import { message } from "@tauri-apps/plugin-dialog";
import { attach } from "./core";
import { fill, t } from "./locales";
import { heicReadable } from "./previews";

export type Shape = "full" | "light";
/** What the person chose: a shape, keeping it as it came, or null when they backed out. */
export type ShapeChoice = Shape | "keep" | null;
type Asking = (said: string) => Promise<ShapeChoice>;

// Photos a window on another computer may not draw; only these for now, as a scan's pages would be lost.
const NOT_EVERYWHERE = ["heic", "heif"];
// Only Windows and macOS carry what turns them; elsewhere they are kept as they came.
const CAN_TURN = ["Macintosh", "Windows"].some((one) => navigator.userAgent.includes(one));

let asking: Asking | null = null;

/** The window asks in its own dress; the system's dialog is only for when no window is there to. */
export const asksShape = (one: Asking | null): void => {
  asking = one;
};

const leaf = (path: string): string =>
  path
    .split(/[\\/]/)
    .filter((one) => one.length > 0)
    .pop() ?? path;

// A path on disk, not a link: a # or ? in a folder's name is part of the name.
const kindOf = (path: string): string => {
  const name = leaf(path);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
};

// A Windows that cannot read HEIC cannot turn it either, so it is not offered there.
export const turnable = (path: string): boolean =>
  CAN_TURN && NOT_EVERYWHERE.includes(kindOf(path)) && heicReadable();

// The system's dialog holds three buttons, so it offers no lighter copy; closing it backs out as Tisty's does.
const askedPlainly: Asking = async (said) => {
  const full = t("reshapeFull");
  const keep = t("reshapeKeep");
  const chosen = await message(said, {
    title: t("reshapeTitle"),
    kind: "info",
    buttons: { yes: full, no: keep, cancel: t("cancel") },
  });
  return chosen === full ? "full" : chosen === keep ? "keep" : null;
};

/** Asked once for every photo dropped together; the core keeps them one at a time while they turn at once. */
export async function attachedAll(paths: string[]): Promise<Promise<string>[]> {
  const asked = paths.filter(turnable);
  if (asked.length === 0) return paths.map((one) => attach(one));
  const said =
    asked.length === 1
      ? fill("reshapeAskOne", leaf(asked[0]))
      : fill("reshapeAskMany", String(asked.length));
  const chosen = await (asking ?? askedPlainly)(said);
  if (chosen === null) return [];
  const shape = chosen === "keep" ? undefined : chosen;
  return paths.map((one) => attach(one, undefined, turnable(one) ? shape : undefined));
}
