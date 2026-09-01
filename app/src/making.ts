import { docNew, docWrite } from "./core";
import { docCard } from "./markdown";

export interface Born {
  id: string;
  said: string;
}

export const spawned = (name: string, folder?: string, pageOf?: string): Promise<Born> => {
  const said = name.trim();
  if (!said) return Promise.reject(new Error("untitled"));
  return docNew(folder, pageOf)
    .then((made) => docWrite(made.id, `# ${said}\n\n`).then(() => made))
    .then((made) => ({ id: made.id, said: docCard(made.id, said) }));
};
