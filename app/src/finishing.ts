import { invoke } from "@tauri-apps/api/core";
import { finished } from "tisty-editor/read";

interface Unfinished {
  name: string;
  html: string;
}

export const finishWhatIsAsked = async (): Promise<void> => {
  try {
    const asked = await invoke<Unfinished[]>("finishing_asked");
    const answers = [];
    for (const { name, html } of asked) answers.push({ name, ...(await finished(html)) });
    await invoke("finishing_answered", { answers });
  } catch {
    await invoke("finishing_answered", { answers: null });
  }
};
