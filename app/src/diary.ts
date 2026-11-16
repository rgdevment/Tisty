import { invoke } from "@tauri-apps/api/core";
import { t } from "./locales";

declare module "./core" {
  interface Filed {
    diary?: boolean;
  }
}

export interface Diary {
  book: string | null;
  open: boolean;
}

export interface Today {
  book: string;
  page: string;
}

export const diaryOf = (): Promise<Diary> => invoke("diary_of");

export const diaryToday = (): Promise<Today> => invoke("diary_today");

export const diaryWrite = (text: string): Promise<Today> => invoke("diary_write", { text });

export const diaryStick = (note: string): Promise<Today> => invoke("diary_stick", { note });

export const diaryAgents = (open: boolean): Promise<void> => invoke("diary_agents", { open });

export const diaryClosed = (file: string, body: string): Promise<string> =>
  invoke("diary_closed", { file, body });

export const diaryDrawn = (file: string, body: string, placed: boolean): Promise<string> =>
  invoke<ArrayBuffer>("diary_drawn", {
    file,
    body,
    placed,
    said: { today: t("diaryToday"), one: t("diaryClosedOne"), many: t("diaryClosedMany") },
  }).then((bytes) => new TextDecoder().decode(bytes));
