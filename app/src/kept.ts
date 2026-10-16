import { saidPlainly } from "./refusal";

export const keptAll = async (tries: Promise<string>[], onError: (said: string) => void) => {
  const done = await Promise.allSettled(tries);
  const failed = done.find((one) => one.status === "rejected");
  if (failed) onError(saidPlainly(failed.reason));
  return done.flatMap((one) => (one.status === "fulfilled" ? [one.value] : [])).join("\n\n");
};

// What fits in a document; a bigger one is refused before the window reads it into memory.
export const PASTED_UP_TO = 650 * 1024 * 1024;
