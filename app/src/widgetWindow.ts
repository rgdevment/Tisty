import { invoke } from "@tauri-apps/api/core";
import type { Viewed } from "./core";

export type WidgetPlace = { readonly at: number; readonly nth: number };

export const viewWidget = (
  source: string,
  from?: Viewed,
  place?: WidgetPlace,
  code = false,
): Promise<void> =>
  invoke("view_widget", {
    source,
    from: from ?? null,
    at: place?.at ?? null,
    nth: place?.nth ?? null,
    code,
  });

export const widgetSave = (now: string): Promise<void> => invoke("widget_save", { now });
