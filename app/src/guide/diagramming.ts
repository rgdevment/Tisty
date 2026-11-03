import { drawing } from "tisty-editor/read";

const sources: string[] = await (await fetch("/sources")).json();
// A label is measured in the font it is shown in, and a face not yet loaded would measure as another.
await document.fonts.load('16px "Inter Variable"');
const drawings = [];
for (const source of sources) {
  drawings.push({
    source,
    light: await drawing(source, false),
    dark: await drawing(source, true),
  });
}
await fetch("/drawn", { method: "POST", body: JSON.stringify(drawings) });
