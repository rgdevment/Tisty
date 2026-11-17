import { invoke } from "@tauri-apps/api/core";
import { t } from "./locales";

declare module "./core" {
  interface Filed {
    diary?: boolean;
  }
}

export interface Diary {
  book: string | null;
  page: string | null;
  day: string;
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

export const diaryFileIdea = (idea: string, heading: string): Promise<Today> =>
  invoke("diary_file_idea", { idea, heading });

export const diaryAgents = (open: boolean): Promise<void> => invoke("diary_agents", { open });

export const diaryNewest = (file: string): Promise<string | null> =>
  invoke("diary_newest", { file });

export const diaryClosed = (file: string, body: string): Promise<string> =>
  invoke("diary_closed", { file, body });

export interface DayAt {
  day: number;
  nth: number;
}

export const diaryLaid = (file: string, body: string, open?: DayAt | null): Promise<string> =>
  invoke<ArrayBuffer>("diary_laid", {
    file,
    body,
    said: {
      one: t("diaryClosedOne"),
      many: t("diaryClosedMany"),
      empty: t("diaryEmpty"),
      writing: t("diaryDayWords"),
      keep: t("diaryKeep"),
      cancel: t("cancel"),
    },
    open: open ?? null,
  }).then((bytes) => new TextDecoder().decode(bytes));

export const diaryDayWords = (body: string, at: DayAt): Promise<string | null> =>
  invoke("diary_day_words", { body, at });

export const diaryRewrite = (file: string, at: DayAt, was: string, words: string): Promise<Today> =>
  invoke("diary_rewrite", { file, at, was, words });

export const diaryUnstick = (
  file: string,
  at: DayAt,
  note: number,
  shown: string,
): Promise<Today> => invoke("diary_unstick", { file, at, note, shown });
