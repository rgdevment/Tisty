import { fell } from "../Standing";

export type LoadedSheet = typeof import("./Sheet").default;

export const sheetLoaded = (): Promise<LoadedSheet> =>
  import("./Sheet").catch(fell).then(({ default: ready }) => ready);
