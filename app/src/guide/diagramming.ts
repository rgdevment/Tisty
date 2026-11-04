import { drawing } from "tisty-editor/read";

export const drawAll = async () => {
  const sources: string[] = await (await fetch("/sources")).json();
  // A label is measured in the font it is shown in, and a face not yet loaded would measure as another.
  await document.fonts.load('16px "Inter Variable"', sources.join(""));
  const drawings = [];
  for (const source of sources) {
    drawings.push({
      source,
      light: await drawing(source, false),
      dark: await drawing(source, true),
    });
  }
  await fetch("/drawn", { method: "POST", body: JSON.stringify(drawings) });
};
