/// A bracket left bare closes the label early, and the reference stops naming anything.
export const labelled = (said: string): string => said.replace(/([[\]\\])/g, "\\$1");

export const bared = (markdown: string): string => {
  const fence = /^\s*(?:```|~~~)/;
  let inside = false;
  return markdown
    .split("\n")
    .map((line) => {
      if (fence.test(line)) {
        inside = !inside;
        return line;
      }
      if (inside) return line;
      return line.replace(/<\/?u\b[^>]*>/gi, "");
    })
    .join("\n");
};
