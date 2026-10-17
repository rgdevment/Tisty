import { docs, docsBrittle, type Filed } from "./core";

export interface Brittle {
  file: string;
  title: string;
  brings: string[];
}

export const scanned = async (
  brittle: () => Promise<string[]> = docsBrittle,
  listed: () => Promise<{ docs: Filed[] }> = docs,
): Promise<Brittle[]> => {
  const found = new Set(await brittle());
  const { docs: all } = await listed();
  return all
    .filter((one) => found.has(one.file))
    .map((one) => ({ file: one.file, title: one.title, brings: ["frailHtml"] }));
};
