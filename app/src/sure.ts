import { ask } from "@tauri-apps/plugin-dialog";

export interface Buttons {
  yes: string;
  no: string;
}

type Asking = (said: string, buttons?: Buttons) => Promise<boolean>;

let asking: Asking | null = null;

export const asksSure = (one: Asking | null): void => {
  asking = one;
};

export const sure = (said: string, buttons?: Buttons): Promise<boolean> => {
  if (asking) return buttons ? asking(said, buttons) : asking(said);
  return ask(said, {
    kind: "warning",
    ...(buttons && { okLabel: buttons.yes, cancelLabel: buttons.no }),
  });
};
