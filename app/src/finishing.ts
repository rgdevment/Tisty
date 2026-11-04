import { invoke } from "@tauri-apps/api/core";
import { finished } from "tisty-editor/read";

interface Unfinished {
  name: string;
  html: string;
}

/** What a window nobody sees was opened for: it finishes what it is asked, says what it did and is closed. Rust looks at the result again before sealing. */
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
