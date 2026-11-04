import { checkHtml } from "@tisty/contracts";
import { finished } from "tisty-editor/read";

const parts = (drawing: Element) => drawing.querySelectorAll("svg, text").length;

const figure = (source: string): string => {
  const held = document.createElement("code");
  held.className = "language-mermaid";
  held.textContent = source;
  return `<figure class="diagram"><pre>${held.outerHTML}</pre></figure>`;
};

/** Finishes each diagram in this browser and answers what the contract says of it. Only `scripts/finished.mjs` loads this. */
export const finishAll = async (): Promise<void> => {
  const sources: string[] = await (await fetch("/sources")).json();
  const answers = [];
  for (const source of sources) {
    const { html, notDrawn } = await finished(figure(source));
    const page = new DOMParser().parseFromString(html, "text/html");
    const [light, dark, ...more] = page.querySelectorAll("figure.diagram > svg");
    // One source says the same in both looks, so another count means one is not the whole drawing. Its box can differ: gitGraph makes up its ids.
    const both =
      !more.length &&
      light?.getAttribute("data-theme") === "light" &&
      dark?.getAttribute("data-theme") === "dark" &&
      parts(light) === parts(dark);
    answers.push({ source, drawn: notDrawn === 0 && both, wrong: checkHtml(html) });
  }
  await fetch("/drawn", { method: "POST", body: JSON.stringify(answers) });
};
