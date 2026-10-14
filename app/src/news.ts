import { invoke } from "@tauri-apps/api/core";
import { locale, t } from "./locales";
import news from "./news.json";

export type Told = { title: string; said: string };

export const newsWaiting = (): Promise<string[]> => invoke("news");
export const newsSeen = (): Promise<void> => invoke("news_seen");

const spanish = (): boolean => locale().toLowerCase().startsWith("es");

export function toldFor(
  versions: string[],
  english = !spanish(),
): { version: string; told: Told[] }[] {
  return versions.flatMap((version) => {
    const found = news.find((one) => one.version === version);
    return found ? [{ version, told: english ? found.en : found.es }] : [];
  });
}

export function headline(version: string | undefined, english = !spanish()): string {
  const named = news.find((one) => one.version === version)?.headline;
  if (named) return english ? named.en : named.es;
  return t("newsTitle");
}
