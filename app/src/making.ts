import { docLink } from "./markdown";
import { docNew, docWrite } from "./core";

export const spawned = (name: string, folder?: string): Promise<string> => {
  const said = name.trim();
  if (!said) return Promise.reject(new Error("untitled"));
  return docNew(folder)
    .then((made) => docWrite(made.id, `# ${said}\n\n`).then(() => made))
    .then((made) => docLink(made.id, said));
};
