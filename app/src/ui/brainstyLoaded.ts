import { fell } from "../Standing";

export type LoadedBrainsty = typeof import("./Brainsty").default;

export const brainstyLoaded = (): Promise<LoadedBrainsty> =>
  import("./Brainsty").catch(fell).then(({ default: ready }) => ready);
