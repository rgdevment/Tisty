import { copied, docPlain, docRead } from "./core";
import { settled } from "./saving";

export const asPlain = async (file: string): Promise<void> => {
  await settled();
  const body = await docPlain(await docRead(file));
  try {
    await copied(body);
  } catch {
    throw { code: "noClipboard" };
  }
};
