import { invoke } from "@tauri-apps/api/core";
import type { Viewed } from "./core";

export const viewDiagram = (
  paper: Viewed,
  at: number,
  nth: number,
  source: string,
): Promise<void> => invoke("view_diagram", { paper, at, nth, source });
export const diagramSave = (now: string): Promise<void> => invoke("diagram_save", { now });
export const diagramAgain = (): Promise<string> => invoke("diagram_again");
export const diagramUnsaved = (unsaved: boolean): Promise<void> =>
  invoke("diagram_unsaved", { unsaved });
export const diagramLeave = (): Promise<void> => invoke("diagram_leave");
export const diagramOrder = (): Promise<number> => invoke("diagram_order");
