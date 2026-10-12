import { ask } from "@tauri-apps/plugin-dialog";

type Asking = (said: string) => Promise<boolean>;

let asking: Asking | null = null;

export const asksSure = (one: Asking | null): void => {
  asking = one;
};

export const sure = (said: string): Promise<boolean> =>
  asking ? asking(said) : ask(said, { kind: "warning" });
