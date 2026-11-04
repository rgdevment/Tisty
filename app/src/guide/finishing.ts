import { checkHtml } from "@tisty/contracts";
import { finished } from "tisty-editor/read";

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
    const themes = [...page.querySelectorAll("figure.diagram > svg")].map((one) =>
      one.getAttribute("data-theme"),
    );
    const both = themes.length === 2 && themes[0] === "light" && themes[1] === "dark";
    answers.push({ source, drawn: notDrawn === 0 && both, wrong: checkHtml(html) });
  }
  await fetch("/drawn", { method: "POST", body: JSON.stringify(answers) });
};
