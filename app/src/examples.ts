import { t } from "./locales";

const BREAK = "\n";

export const ahead = (): string[] => t("taskExamplesAhead").split(BREAK);

export const repeating = (): string[] => t("taskExamplesRepeating").split(BREAK);

export const every = (): string[] => [
  ...ahead(),
  ...repeating(),
  ...t("taskExamplesElse").split(BREAK),
];

// Filing an example that names a list nobody has would make that list.
export const unlisted = (all: string[]): string[] => all.filter((one) => !one.includes("@"));

export const turnOf = (all: string[], at: number): string => all[Math.floor(at * all.length)] ?? "";
