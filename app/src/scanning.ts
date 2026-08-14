import { docRead, docs, type Filed } from "./core";
import { frail } from "./frail";

export interface Brittle {
  file: string;
  title: string;
  brings: string[];
}

export const scanned = async (
  read: (file: string) => Promise<string> = docRead,
  listed: () => Promise<{ docs: Filed[] }> = docs,
): Promise<Brittle[]> => {
  const { docs: all } = await listed();
  const found: Brittle[] = [];
  for (const one of all) {
    const brings = frail(await read(one.file));
    if (brings.length) found.push({ file: one.file, title: one.title, brings });
  }
  return found;
};
