export interface Head {
  key: string;
  level: number;
  text: string;
  go: () => void;
}

// Level 0 is no heading: it opens the properties, which the page keeps out of sight.
export const heads = (page: Document): Head[] => {
  const front = page.querySelector<HTMLElement>(".page > .front:first-child");
  const opening: Head[] = front
    ? [{ key: "front", level: 0, text: "", go: () => front.click() }]
    : [];
  return opening.concat(listedHeads(page));
};

const listedHeads = (page: Document): Head[] =>
  Array.from(
    page.querySelectorAll<HTMLElement>(".page > :is(h1, h2, h3, h4, h5, h6):not([data-title])"),
  ).map((one, at) => ({
    key: `${at}:${one.id}`,
    level: Number(one.tagName.slice(1)),
    text: one.textContent ?? "",
    go: () => one.scrollIntoView({ block: "start" }),
  }));
