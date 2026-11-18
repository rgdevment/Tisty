import { copied, docAddress, docPlain, docRead } from "./core";
import { settled } from "./saving";

const taken = async (text: string): Promise<void> => {
  try {
    await copied(text);
  } catch {
    throw { code: "noClipboard" };
  }
};

export const asPlain = async (file: string): Promise<void> => {
  await settled();
  await taken(await docPlain(await docRead(file)));
};

export const asLink = async (file: string): Promise<void> => taken(await docAddress(file));
