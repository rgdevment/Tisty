import { copied, docRead } from "./core";
import { settled } from "./saving";
import { bared } from "./ui/writing";

export const asPlain = async (file: string): Promise<void> => {
  await settled();
  const body = bared(await docRead(file));
  try {
    await copied(body);
  } catch {
    throw { code: "noClipboard" };
  }
};
